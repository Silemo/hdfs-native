#[cfg(feature = "integration-test")]
mod common;

#[cfg(feature = "integration-test")]
mod test {
    use log::debug;
    use crate::common::{assert_bufs_equal, setup, TEST_FILE_INTS};
    use bytes::{BufMut, BytesMut};
    use hdfs_native::{client::FileStatus, minidfs::DfsFeatures, Client, Result, WriteOptions};
    use serial_test::serial;
    use std::collections::{HashSet, HashMap};

    #[tokio::test]
    #[serial]
    async fn test_basic() {
        test_with_features(&HashSet::new()).await.unwrap();
    }

    // #[tokio::test]
    // #[serial]
    // async fn test_security_kerberos() {
    //     test_with_features(&HashSet::from([DfsFeatures::Security]))
    //         .await
    //         .unwrap();
    // }

    // #[tokio::test]
    // #[serial]
    // async fn test_security_token() {
    //     test_with_features(&HashSet::from([DfsFeatures::Security, DfsFeatures::Token]))
    //         .await
    //         .unwrap();
    // }

    // #[tokio::test]
    // #[serial]
    // async fn test_integrity_kerberos() {
    //     test_with_features(&HashSet::from([
    //         DfsFeatures::Security,
    //         DfsFeatures::Integrity,
    //     ]))
    //     .await
    //     .unwrap();
    // }

    // #[tokio::test]
    // #[serial]
    // async fn test_integrity_token() {
    //     test_with_features(&HashSet::from([
    //         DfsFeatures::Security,
    //         DfsFeatures::Token,
    //         DfsFeatures::Integrity,
    //     ]))
    //     .await
    //     .unwrap();
    // }

    // #[tokio::test]
    // #[serial]
    // async fn test_privacy_kerberos() {
    //     test_with_features(&HashSet::from([
    //         DfsFeatures::Security,
    //         DfsFeatures::Privacy,
    //     ]))
    //     .await
    //     .unwrap();
    // }

    // #[tokio::test]
    // #[serial]
    // async fn test_privacy_token() {
    //     test_with_features(&HashSet::from([
    //         DfsFeatures::Security,
    //         DfsFeatures::Token,
    //         DfsFeatures::Privacy,
    //     ]))
    //     .await
    //     .unwrap();
    // }

    // #[tokio::test]
    // #[serial]
    // async fn test_aes() {
    //     test_with_features(&HashSet::from([
    //         DfsFeatures::Security,
    //         DfsFeatures::Privacy,
    //         DfsFeatures::AES,
    //     ]))
    //     .await
    //     .unwrap();
    // }

    // #[tokio::test]
    // #[serial]
    // async fn test_forced_data_transfer_encryption() {
    //     // DataTransferEncryption enabled but privacy isn't, still force encryption
    //     test_with_features(&HashSet::from([
    //         DfsFeatures::Security,
    //         DfsFeatures::DataTransferEncryption,
    //     ]))
    //     .await
    //     .unwrap();
    // }

    // #[tokio::test]
    // #[serial]
    // async fn test_data_transfer_encryption() {
    //     test_with_features(&HashSet::from([
    //         DfsFeatures::Security,
    //         DfsFeatures::Privacy,
    //         DfsFeatures::DataTransferEncryption,
    //     ]))
    //     .await
    //     .unwrap();
    // }

    // #[tokio::test]
    // #[serial]
    // async fn test_data_transfer_encryption_aes() {
    //     test_with_features(&HashSet::from([
    //         DfsFeatures::Security,
    //         DfsFeatures::Privacy,
    //         DfsFeatures::DataTransferEncryption,
    //         DfsFeatures::AES,
    //     ]))
    //     .await
    //     .unwrap();
    // }

    // #[tokio::test]
    // #[serial]
    // async fn test_basic_ha() {
    //     test_with_features(&HashSet::from([DfsFeatures::HA]))
    //         .await
    //         .unwrap();
    // }

    // #[tokio::test]
    // #[serial]
    // async fn test_security_privacy_ha() {
    //     test_with_features(&HashSet::from([
    //         DfsFeatures::Security,
    //         DfsFeatures::Privacy,
    //         DfsFeatures::HA,
    //     ]))
    //     .await
    //     .unwrap();
    // }

    // #[tokio::test]
    // #[serial]
    // async fn test_security_token_ha() {
    //     test_with_features(&HashSet::from([
    //         DfsFeatures::Security,
    //         DfsFeatures::Token,
    //         DfsFeatures::HA,
    //     ]))
    //     .await
    //     .unwrap();
    // }

    // #[tokio::test]
    // #[serial]
    // async fn test_rbf() {
    //     test_with_features(&HashSet::from([DfsFeatures::RBF]))
    //         .await
    //         .unwrap();
    // }

    pub async fn test_with_features(features: &HashSet<DfsFeatures>) -> Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        //let _dfs = setup(features);
        let mut config: HashMap<String, String> = HashMap::new();
        
        let client = Client::new_with_config("hdfs://rpc.namenode.service.consul:8020/tmp/integration-tests", config).unwrap();
        test_file_info(&client).await?;
        test_listing(&client).await?;
        test_rename(&client).await?;
        test_dirs(&client).await?;
        test_read_write(&client).await?;
        // We use writing to create files, so do this after
        test_recursive_listing(&client).await?;
        test_set_times(&client).await?;
        test_set_owner(&client).await?;
        test_set_permission(&client).await?;
        test_set_replication(&client).await?;
        test_get_content_summary(&client).await?;

        Ok(())
    }

    async fn test_file_info(client: &Client) -> Result<()> {
        let status = client.get_file_info("/tmp/integration-tests/testfile").await?;
        // Path is empty, I guess because we already know what file we just got the info for?
        assert_eq!(status.path, "/tmp/integration-tests/testfile");
        assert_eq!(status.length, TEST_FILE_INTS * 4);
        Ok(())
    }

    async fn test_listing(client: &Client) -> Result<()> {
        let statuses: Vec<FileStatus> = client
            .list_status("/tmp/integration-tests/", false)
            .await?
            .into_iter()
            // Only include files, since federation things could result in folders being created
            .filter(|s| !s.isdir)
            .collect();
        assert_eq!(statuses.len(), 1);
        let status = &statuses[0];
        assert_eq!(status.path, "/tmp/integration-tests/testfile");
        assert_eq!(status.length, TEST_FILE_INTS * 4);
        Ok(())
    }

    async fn test_rename(client: &Client) -> Result<()> {
        client.rename("/tmp/integration-tests/testfile", "/tmp/integration-tests/testfile2", false).await?;

        assert!(client.list_status("/tmp/integration-tests/testfile", false).await.is_err());
        assert_eq!(client.list_status("/tmp/integration-tests/testfile2", false).await?.len(), 1);

        client.rename("/tmp/integration-tests/testfile2", "/tmp/integration-tests/testfile", false).await?;
        assert!(client.list_status("/tmp/integration-tests/testfile2", false).await.is_err());
        assert_eq!(client.list_status("/tmp/integration-tests/testfile", false).await?.len(), 1);

        Ok(())
    }

    async fn test_dirs(client: &Client) -> Result<()> {
        client.mkdirs("/tmp/integration-tests/testdir", 0o755, false).await?;
        assert!(client
            .list_status("/tmp/integration-tests/testdir", false)
            .await
            .is_ok_and(|s| s.is_empty()));

        client.delete("/tmp/integration-tests/testdir", false).await?;
        assert!(client.list_status("/tmp/integration-tests/testdir", false).await.is_err());

        client.mkdirs("/tmp/integration-tests/testdir1/testdir2", 0o755, true).await?;
        assert!(client
            .list_status("/tmp/integration-tests/testdir1", false)
            .await
            .is_ok_and(|s| s.len() == 1));

        // Deleting non-empty dir without recursive fails
        assert!(client.delete("/tmp/integration-tests/testdir1", false).await.is_err());
        assert!(client.delete("/tmp/integration-tests/testdir1", true).await.is_ok_and(|r| r));

        Ok(())
    }

    async fn test_read_write(client: &Client) -> Result<()> {
        let write_options = WriteOptions::default().overwrite(true);

        // Create an empty file
        let mut writer = client.create("/tmp/integration-tests/newfile", &write_options).await?;

        writer.close().await?;

        assert_eq!(client.get_file_info("/tmp/integration-tests/newfile").await?.length, 0);

        let mut writer = client.create("/tmp/integration-tests/newfile", &write_options).await?;

        let mut file_contents = BytesMut::new();
        let mut data = BytesMut::new();
        for i in 0..1024 {
            file_contents.put_i32(i);
            data.put_i32(i);
        }

        let buf = data.freeze();

        writer.write(buf).await?;
        writer.close().await?;

        assert_eq!(client.get_file_info("/tmp/integration-tests/newfile").await?.length, 4096);

        let mut reader = client.read("/tmp/integration-tests/newfile").await?;
        let read_data = reader.read(reader.file_length()).await?;

        assert_bufs_equal(&file_contents, &read_data, None);

        let mut data = BytesMut::new();
        for i in 0..1024 {
            file_contents.put_i32(i);
            data.put_i32(i);
        }

        let buf = data.freeze();

        let mut writer = client.append("/tmp/integration-tests/newfile").await?;
        writer.write(buf).await?;
        writer.close().await?;

        let mut reader = client.read("/tmp/integration-tests/newfile").await?;
        let read_data = reader.read(reader.file_length()).await?;

        assert_bufs_equal(&file_contents, &read_data, None);

        assert!(client.delete("/tmp/integration-tests/newfile", false).await.is_ok_and(|r| r));

        Ok(())
    }

    async fn test_recursive_listing(client: &Client) -> Result<()> {
        let write_options = WriteOptions::default();
        client.mkdirs("/tmp/integration-tests/dir/nested", 0o755, true).await?;
        client
            .create("/tmp/integration-tests/dir/file1", &write_options)
            .await?
            .close()
            .await?;
        client
            .create("/tmp/integration-tests/dir/nested/file2", &write_options)
            .await?
            .close()
            .await?;
        client
            .create("/tmp/integration-tests/dir/nested/file3", &write_options)
            .await?
            .close()
            .await?;

        let statuses = client.list_status("/tmp/integration-tests/dir", true).await?;
        assert_eq!(statuses.len(), 4);

        client.delete("/tmp/integration-tests/dir", true).await?;

        Ok(())
    }

    async fn test_set_times(client: &Client) -> Result<()> {
        client
            .create("/tmp/integration-tests/test", WriteOptions::default())
            .await?
            .close()
            .await?;

        let mtime = 1717641455;
        let atime = 1717641456;

        client.set_times("/tmp/integration-tests/test", mtime, atime).await?;

        let file_info = client.get_file_info("/tmp/integration-tests/test").await?;

        assert_eq!(file_info.modification_time, mtime);
        assert_eq!(file_info.access_time, atime);

        client.delete("/tmp/integration-tests/test", false).await?;

        Ok(())
    }

    async fn test_set_owner(client: &Client) -> Result<()> {
        client
            .create("/tmp/integration-tests/test", WriteOptions::default())
            .await?
            .close()
            .await?;

        client
            .set_owner("/tmp/integration-tests/test", Some("testuser"), Some("testgroup"))
            .await?;
        let file_info = client.get_file_info("/tmp/integration-tests/test").await?;

        assert_eq!(file_info.owner, "testuser");
        assert_eq!(file_info.group, "testgroup");

        client.set_owner("/tmp/integration-tests/test", Some("testuser2"), None).await?;
        let file_info = client.get_file_info("/tmp/integration-tests/test").await?;

        assert_eq!(file_info.owner, "testuser2");
        assert_eq!(file_info.group, "testgroup");

        client.set_owner("/tmp/integration-tests/test", None, Some("testgroup2")).await?;
        let file_info = client.get_file_info("/tmp/integration-tests/test").await?;

        assert_eq!(file_info.owner, "testuser2");
        assert_eq!(file_info.group, "testgroup2");

        client.delete("/tmp/integration-tests/test", false).await?;

        Ok(())
    }

    async fn test_set_permission(client: &Client) -> Result<()> {
        client
            .create("/tmp/integration-tests/test", WriteOptions::default())
            .await?
            .close()
            .await?;

        let file_info = client.get_file_info("/tmp/integration-tests/test").await?;
        assert_eq!(file_info.permission, 0o644);

        client.set_permission("/tmp/integration-tests/test", 0o600).await?;
        let file_info = client.get_file_info("/tmp/integration-tests/test").await?;
        assert_eq!(file_info.permission, 0o600);

        client.delete("/tmp/integration-tests/test", false).await?;

        Ok(())
    }

    async fn test_set_replication(client: &Client) -> Result<()> {
        client
            .create("/tmp/integration-tests/test", WriteOptions::default())
            .await?
            .close()
            .await?;

        client.set_replication("/tmp/integration-tests/test", 1).await?;
        let file_info = client.get_file_info("/tmp/integration-tests/test").await?;
        assert_eq!(file_info.replication, Some(1));

        client.set_replication("/tmp/integration-tests/test", 2).await?;
        let file_info = client.get_file_info("/tmp/integration-tests/test").await?;
        assert_eq!(file_info.replication, Some(2));

        client.delete("/tmp/integration-tests/test", false).await?;

        Ok(())
    }

    async fn test_get_content_summary(client: &Client) -> Result<()> {
        let mut file1 = client.create("/tmp/integration-tests/test", WriteOptions::default()).await?;

        file1.write(vec![0, 1, 2, 3].into()).await?;
        file1.close().await?;

        let mut file2 = client.create("/tmp/integration-tests/test2", WriteOptions::default()).await?;

        file2.write(vec![0, 1, 2, 3, 4, 5].into()).await?;
        file2.close().await?;

        client.mkdirs("/tmp/integration-tests/testdir", 0o755, true).await?;

        let content_summary = client.get_content_summary("/tmp/integration-tests/").await?;
        assert_eq!(content_summary.file_count, 3,);
        assert_eq!(content_summary.directory_count, 2);
        // Test file plus the two we made above
        assert_eq!(content_summary.length, TEST_FILE_INTS as u64 * 4 + 4 + 6);

        Ok(())
    }
}
