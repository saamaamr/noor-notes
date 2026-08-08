import Gio from 'gi://Gio';
import GLib from 'gi://GLib';

import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

import {MotionSession} from './motionSession.js';

const PowerProfilesIface = `<node>
<interface name="net.hadess.PowerProfiles">
    <property name="ActiveProfile" type="s" access="readwrite"/>
</interface>
</node>`;

const PowerProfilesProxy = Gio.DBusProxy.makeProxyWrapper(PowerProfilesIface);

export default class NoorLockscreenMotionExtension extends Extension {
    enable() {
        this._motion = new MotionSession();
        this._interfaceSettings = new Gio.Settings({
            schema_id: 'org.gnome.desktop.interface',
        });
        this._screenShieldSignalId = 0;
        this._animationsSignalId = 0;
        this._powerProfilesSignalId = 0;
        this._idleId = 0;
        this._activationSerial = 0;
        this._powerProfilesProxy = null;

        this._screenShieldSignalId = Main.screenShield.connect('active-changed',
            () => this._syncLockState());
        this._animationsSignalId = this._interfaceSettings.connect(
            'changed::enable-animations', () => this._refreshPolicy());

        try {
            this._powerProfilesProxy = new PowerProfilesProxy(
                Gio.DBus.system,
                'net.hadess.PowerProfiles',
                '/net/hadess/PowerProfiles',
                (proxy, error) => {
                    if (error) {
                        console.warn(`Noor Lockscreen Motion: power profile unavailable: ${error.message}`);
                        return;
                    }
                    this._powerProfilesSignalId = proxy.connect(
                        'g-properties-changed', () => this._refreshPolicy());
                    this._refreshPolicy();
                });
        } catch (error) {
            console.warn(`Noor Lockscreen Motion: could not initialize power profile support: ${error.message}`);
        }

        this._syncLockState();
    }

    disable() {
        this._removeIdle();

        if (this._screenShieldSignalId) {
            Main.screenShield.disconnect(this._screenShieldSignalId);
            this._screenShieldSignalId = 0;
        }
        if (this._animationsSignalId) {
            this._interfaceSettings?.disconnect(this._animationsSignalId);
            this._animationsSignalId = 0;
        }
        if (this._powerProfilesSignalId) {
            this._powerProfilesProxy?.disconnect(this._powerProfilesSignalId);
            this._powerProfilesSignalId = 0;
        }

        this._motion.stop();
        this._motion = null;
        this._interfaceSettings = null;
        this._powerProfilesProxy = null;
    }

    _syncLockState() {
        this._removeIdle();
        if (!Main.screenShield.active) {
            this._motion.stop();
            return;
        }

        const activationKey = ++this._activationSerial;
        this._idleId = GLib.idle_add(GLib.PRIORITY_DEFAULT_IDLE, () => {
            this._idleId = 0;
            try {
                this._motion?.start({
                    screenShield: Main.screenShield,
                    animationsEnabled: this._animationsEnabled(),
                    powerProfile: this._activePowerProfile(),
                    activationKey,
                });
            } catch (error) {
                console.warn(`Noor Lockscreen Motion: lock animation skipped: ${error.message}`);
                this._motion?.stop();
            }
            return GLib.SOURCE_REMOVE;
        });
    }

    _refreshPolicy() {
        this._motion?.refreshPolicy({
            animationsEnabled: this._animationsEnabled(),
            powerProfile: this._activePowerProfile(),
        });
    }

    _animationsEnabled() {
        return this._interfaceSettings?.get_boolean('enable-animations') ?? true;
    }

    _activePowerProfile() {
        return this._powerProfilesProxy?.ActiveProfile ?? 'balanced';
    }

    _removeIdle() {
        if (!this._idleId)
            return;
        GLib.Source.remove(this._idleId);
        this._idleId = 0;
    }
}
