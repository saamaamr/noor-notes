use noor_sync::{BackupObject, BackupProvider, GoogleDriveProvider, OneDriveProvider};
use wiremock::matchers::{body_bytes, body_string_contains, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn google_file(id: &str, name: &str, size: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": name,
        "modifiedTime": "2026-09-03T12:00:00Z",
        "size": size.to_string()
    })
}

fn drive_item(id: &str, name: &str, size: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": name,
        "lastModifiedDateTime": "2026-09-03T12:00:00Z",
        "size": size
    })
}

#[tokio::test]
async fn google_uses_app_data_and_stages_before_updating_current() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/files"))
        .and(body_string_contains("current.nnbackup.upload"))
        .and(body_string_contains("appDataFolder"))
        .respond_with(ResponseTemplate::new(200).set_body_json(google_file(
            "temporary",
            "current.nnbackup.upload",
            0,
        )))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/files/temporary"))
        .and(query_param("uploadType", "media"))
        .and(header("authorization", "Bearer access"))
        .and(body_bytes(b"encrypted bytes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(google_file(
            "temporary",
            "current.nnbackup.upload",
            15,
        )))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/files"))
        .and(query_param("spaces", "appDataFolder"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"files": []})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/files"))
        .and(body_string_contains("current.nnbackup"))
        .respond_with(ResponseTemplate::new(200).set_body_json(google_file(
            "current",
            "current.nnbackup",
            0,
        )))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/files/current"))
        .and(body_bytes(b"encrypted bytes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(google_file(
            "current",
            "current.nnbackup",
            15,
        )))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/files/temporary"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let provider = GoogleDriveProvider::for_test(&server.uri()).unwrap();
    let object = provider
        .upload("access", "current.nnbackup", b"encrypted bytes")
        .await
        .unwrap();

    assert_eq!(object.name, "current.nnbackup");
    assert_eq!(object.size, 15);
}

#[tokio::test]
async fn onedrive_uses_only_approot_paths_for_upload_list_download_and_delete() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/me/drive/special/approot/children"))
        .and(body_string_contains("Noor Notes"))
        .respond_with(ResponseTemplate::new(201))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path(
            "/me/drive/special/approot:/Noor%20Notes/current.nnbackup:/content",
        ))
        .and(header("authorization", "Bearer access"))
        .and(body_bytes(b"ciphertext"))
        .respond_with(ResponseTemplate::new(200).set_body_json(drive_item(
            "object-one",
            "current.nnbackup",
            10,
        )))
        .expect(1)
        .mount(&server)
        .await;
    let provider = OneDriveProvider::for_test(&server.uri()).unwrap();
    let object = provider
        .upload("access", "current.nnbackup", b"ciphertext")
        .await
        .unwrap();
    assert_eq!(object.id, "object-one");

    Mock::given(method("GET"))
        .and(path("/me/drive/special/approot:/Noor%20Notes:/children"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "value": [drive_item("object-one", "current.nnbackup", 10)]
        })))
        .expect(1)
        .mount(&server)
        .await;
    assert_eq!(provider.list("access").await.unwrap(), vec![object.clone()]);

    Mock::given(method("GET"))
        .and(path("/me/drive/items/object-one/content"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"ciphertext"))
        .expect(1)
        .mount(&server)
        .await;
    assert_eq!(
        provider.download("access", &object).await.unwrap(),
        b"ciphertext"
    );

    Mock::given(method("DELETE"))
        .and(path("/me/drive/items/object-one"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    provider.delete("access", &object).await.unwrap();
}

#[tokio::test]
async fn unsafe_names_large_downloads_and_redirects_are_rejected() {
    let server = MockServer::start().await;
    let provider = OneDriveProvider::for_test(&server.uri()).unwrap();
    assert!(
        provider
            .upload("access", "../notes", b"ciphertext")
            .await
            .is_err()
    );

    let object = BackupObject {
        id: "large".into(),
        name: "current.nnbackup".into(),
        modified_at: chrono::DateTime::from_timestamp(0, 0).unwrap(),
        size: 128_u64 * 1024 * 1024 + 1,
    };
    Mock::given(method("GET"))
        .and(path("/me/drive/items/large/content"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Length", (128_u64 * 1024 * 1024 + 1).to_string()),
        )
        .mount(&server)
        .await;
    assert!(provider.download("access", &object).await.is_err());

    let redirect = BackupObject {
        id: "redirect".into(),
        size: 0,
        ..object
    };
    Mock::given(method("GET"))
        .and(path("/me/drive/items/redirect/content"))
        .respond_with(
            ResponseTemplate::new(302).insert_header("Location", "https://evil.example/file"),
        )
        .mount(&server)
        .await;
    assert!(provider.download("access", &redirect).await.is_err());
}
