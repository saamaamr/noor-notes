import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';

import {authorizeRequest} from './policy.js';

const BUS_NAME = 'io.github.saamaamr.NoorNotes.Windowing';
const APP_BUS_NAME = 'io.github.saamaamr.NoorNotes';
const OBJECT_PATH = '/io/github/saamaamr/NoorNotes/Window1';
const INTERFACE_XML = `
<node>
  <interface name="io.github.saamaamr.NoorNotes.Window1">
    <method name="SetAbove">
      <arg type="s" name="window_id" direction="in"/>
      <arg type="b" name="enabled" direction="in"/>
    </method>
    <method name="SetAllWorkspaces">
      <arg type="s" name="window_id" direction="in"/>
      <arg type="b" name="enabled" direction="in"/>
    </method>
  </interface>
</node>`;

class WindowService {
    constructor() {
        this._changed = new Set();
    }

    SetAboveAsync([windowId, enabled], invocation) {
        this._apply('SetAbove', windowId, enabled, invocation, window => {
            if (enabled)
                window.make_above();
            else
                window.unmake_above();
        });
    }

    SetAllWorkspacesAsync([windowId, enabled], invocation) {
        this._apply('SetAllWorkspaces', windowId, enabled, invocation, window => {
            if (enabled)
                window.stick();
            else
                window.unstick();
        });
    }

    restore() {
        for (const window of this._changed) {
            if (!window)
                continue;
            window.unmake_above();
            window.unstick();
        }
        this._changed.clear();
    }

    _apply(method, windowId, enabled, invocation, operation) {
        const sender = invocation.get_sender();
        const owner = this._appBusOwner();
        const window = this._findWindow(windowId);
        const appId = window?.get_gtk_application_id?.() ?? '';
        if (!window || !authorizeRequest({method, windowId, enabled, appId, sender, owner, stale: !window.get_compositor_private()})) {
            invocation.return_dbus_error(
                'io.github.saamaamr.NoorNotes.Window1.NotAuthorized',
                'The request is not authorized for this window');
            return;
        }
        operation(window);
        this._changed.add(window);
        invocation.return_value(new GLib.Variant('()', []));
    }

    _findWindow(windowId) {
        return global.get_window_actors()
            .map(actor => actor.meta_window)
            .find(window => window.get_title() === windowId) ?? null;
    }

    _appBusOwner() {
        try {
            const result = Gio.DBus.session.call_sync(
                'org.freedesktop.DBus',
                '/org/freedesktop/DBus',
                'org.freedesktop.DBus',
                'GetNameOwner',
                new GLib.Variant('(s)', [APP_BUS_NAME]),
                new GLib.VariantType('(s)'),
                Gio.DBusCallFlags.NONE,
                -1,
                null);
            return result.deepUnpack()[0];
        } catch (_error) {
            return '';
        }
    }
}

export default class NoorNotesWindowingExtension extends Extension {
    enable() {
        this._service = new WindowService();
        this._exported = Gio.DBusExportedObject.wrapJSObject(INTERFACE_XML, this._service);
        this._exported.export(Gio.DBus.session, OBJECT_PATH);
        this._ownerId = Gio.bus_own_name(
            Gio.BusType.SESSION,
            BUS_NAME,
            Gio.BusNameOwnerFlags.NONE,
            null,
            null);
    }

    disable() {
        this._service?.restore();
        this._exported?.unexport();
        if (this._ownerId)
            Gio.bus_unown_name(this._ownerId);
        this._ownerId = null;
        this._exported = null;
        this._service = null;
    }
}
