use axum::{body::Bytes, http::StatusCode, BoxError};
use futures::{Stream, TryStreamExt};
use tokio::{
    fs::File,
    io::{self, BufWriter},
};
use tokio_util::io::StreamReader;

const UPLOADS_DIRECTORY: &str = "upload";

pub async fn stream_to_file<S, E>(path: &str, stream: S) -> Result<(), (StatusCode, String)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    if !path_is_valid(path) {
        return Err((StatusCode::BAD_REQUEST, "Invalid path".to_owned()));
    }

    return async {
        // convert the stream into an `AsyncRead`
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        // create the file
        let path = std::path::Path::new(UPLOADS_DIRECTORY).join(path);
        tracing::info!("{:?}", path);
        let mut file = BufWriter::new(File::create(path).await?);

        // copy the body into the file!
        tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok::<_, io::Error>(())
    }
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()));
}

// prevent directory travarsal attacks
// ensure the path consists of exactly
// one normal component
fn path_is_valid(path: &str) -> bool {
    let path = std::path::Path::new(path);
    let mut components = path.components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }

    return components.count() == 1;
}
