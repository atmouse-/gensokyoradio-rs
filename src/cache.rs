use std::{path::PathBuf, path::Path, io};
use url::Url;
use reqwest;
use std::io::Cursor;
use tokio;
use tokio::runtime::Runtime;

pub struct CacheDir {
    path: PathBuf,
}

async fn fetch_url(url: String, file_name: String) -> Result<(), ()> {
    let response = reqwest::get(url).await.unwrap();
    let mut file = std::fs::File::create(&file_name).unwrap();
    let mut content =  Cursor::new(response.bytes().await.unwrap());
    std::io::copy(&mut content, &mut file).unwrap();
    Ok(())
}

impl CacheDir {
    pub fn init() -> CacheDir {
        // TODO: check if dir exist

        let cache_dir = dirs::cache_dir().unwrap().join("gensokyoradio");
        if ! &cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir).unwrap();
        }

        if ! &cache_dir.is_dir() {
            // TODO:
            error!("error cache_dir: {:?} not a directory", &cache_dir);
        }

        CacheDir {
            path: cache_dir
        }
    }

    fn fetch(&self, url: Url, local_file: &PathBuf) -> Result<(),()> {
        // TODO: timeout tokio task
        tokio::task::spawn(
            fetch_url(url.to_string(), local_file.to_str().unwrap().to_owned())
        );
        Ok(())

    }

    fn fetch_pending(&self, url: Url, local_file: &PathBuf) -> Result<(),()> {
        let mut rt = Runtime::new().unwrap();
        rt.block_on(
            fetch_url(url.to_string(), local_file.to_str().unwrap().to_owned())
        );
        Ok(())
    }

    pub fn join(&self, cache_file: &str) -> String {
        self.path.join(cache_file).to_str().unwrap().to_owned()
    }

    pub fn hash(&self, albumart: &str) -> Result<PathBuf, io::Error> {
        let url_albumart = url::Url::parse(albumart).unwrap();
        let url_path = url_albumart.path().strip_prefix("/").unwrap();
        let local_file = self.path.join(url_path);
        let local_path = 
            &local_file.parent().unwrap().to_path_buf();

        if ! &local_path.exists() {
            std::fs::create_dir_all(&local_path).unwrap();
            debug!("cache dir: {} generated", &local_path.to_str().unwrap());
        }

        // TODO: robust
        if ! local_file.exists() {
            self.fetch_pending(url_albumart, &local_file).unwrap();
        };

        // TODO: valid file checking
        //
        debug!("local_file: {:?}", &local_file);
        Ok(local_file)
    }

}

