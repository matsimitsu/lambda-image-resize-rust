extern crate aws_lambda_events;
extern crate image;
#[macro_use]
extern crate lambda_runtime as lambda;
#[macro_use]
extern crate log;
extern crate rayon;
extern crate s3;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate simple_logger;

use image::{ImageOutputFormat, GenericImageView, ImageError};

use rayon::prelude::*;
use s3::bucket::Bucket;
use s3::credentials::Credentials;
use s3::region::Region;

mod config;

use config::Config;
use lambda::error::HandlerError;
use serde_json::Value;
use std::error::Error;
use aws_lambda_events::event::apigw::ApiGatewayProxyRequest;

const BUCKET_KEY: &'static str = "bucket";
const FILE_PATH_KEY: &'static str = "key";
const REGION_KEY: &'static str = "region";
const SIZE_KEY: &'static str = "size";

fn main() -> Result<(), Box<Error>> {
    simple_logger::init_with_level(log::Level::Info)?;

    lambda!(handle_event);

    Ok(())
}

fn handle_event(event: Value, ctx: lambda::Context) -> Result<(), HandlerError> {
    let config = Config::new();

    let api_event: ApiGatewayProxyRequest = serde_json::from_value(event).map_err(|e| ctx.new_error(e.to_string().as_str()))?;

    let bucket = api_event.query_string_parameters.get(BUCKET_KEY).unwrap_or_else(|| panic!("Missing bucket"));
    let file_key = api_event.query_string_parameters.get(FILE_PATH_KEY).unwrap_or_else(|| panic!("Missing file key"));
    let region = api_event.query_string_parameters.get(REGION_KEY).unwrap_or_else(|| panic!("Missing region"));
    let size = api_event.query_string_parameters.get(SIZE_KEY).unwrap_or_else(|| panic!("Missing size"));

    info!("Bucket: {}, key: {}, region: {}", &bucket, &file_key, &region);
    handle_request(&config, bucket.to_string(), file_key.to_string(), region.to_string(), size.to_string());
    Ok(())
}

fn handle_request(config: &Config, bucket_name: String, file_path: String, region_name: String, size_as_string: String) {
    let size = size_as_string.parse::<f32>();
    let credentials = Credentials::default();
    let region: Region = region_name.parse().unwrap();
    let bucket = Bucket::new(&bucket_name, region, credentials);
    info!("Successfully authenticated");

//    let actual_size = check_size(size, &config);

    let (data, _) = bucket
        .get(&file_path)
        .expect(&format!("Could not get object: {}", &file_path));

    let img = image::load_from_memory(&data)
        .ok()
        .expect("Opening image failed");


    let buffer = resize_image(&img, &size).expect("Could not resize image");

    let mut target = file_path.clone();

    target = format!("{t}-resize-{s}", t=target, s=size);

    let (_, code) = bucket
        .put(&target, &buffer, "image/jpeg")
        .expect(&format!("Could not upload object to :{}", &target));
    info!("Uploaded: {} with: {}", &target, &code);
}

fn resize_image(img: &image::DynamicImage, new_w: &f32) -> Result<Vec<u8>, ImageError> {
    let mut result: Vec<u8> = Vec::new();

    let old_w = img.width() as f32;
    let old_h = img.height() as f32;
    let ratio = new_w / old_w;
    let new_h = (old_h * ratio).floor();

    let scaled = img.resize(*new_w as u32, new_h as u32, image::FilterType::Lanczos3);
    scaled.write_to(&mut result, ImageOutputFormat::JPEG(90))?;

    Ok(result)
}

//fn check_size(required_size: String, config: &Config) -> f32 {
//    for allowed_size in &config.sizes {
//        if format!("{}", allowed_size).eq(required_size) {
//           return allowed_size.;
//        }
//    }
//}
