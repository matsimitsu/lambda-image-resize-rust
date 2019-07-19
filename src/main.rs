extern crate aws_lambda_events;
extern crate image;
#[macro_use]
extern crate lambda_runtime as lambda;
#[macro_use]
extern crate log;
extern crate rayon;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate simple_logger;
extern crate reqwest;

use image::{ImageOutputFormat, GenericImageView, ImageError};


mod config;

use config::Config;
use lambda::error::HandlerError;
use serde_json::Value;
use std::error::Error;
use aws_lambda_events::event::apigw::ApiGatewayProxyRequest;
use aws_lambda_events::event::apigw::ApiGatewayProxyResponse;
use std::collections::HashMap;
use std::io::Read;


const SIZE_KEY: &'static str = "size";

const SOURCE_HEADER: &'static str = "source-url";
const DEST_HEADER: &'static str = "destination-url";


fn main() -> Result<(), Box<Error>> {
    simple_logger::init_with_level(log::Level::Info)?;

    let response = lambda!(handle_event);
    Ok(response)
}

fn handle_event(event: Value, ctx: lambda::Context) -> Result<ApiGatewayProxyResponse, HandlerError> {
    let config = Config::new();

    let api_event: ApiGatewayProxyRequest = serde_json::from_value(event).map_err(|e| ctx.new_error(e.to_string().as_str()))?;

    let source_url = api_event.headers.get(SOURCE_HEADER).unwrap_or_else(|| panic!("Missing source url"));
    let dest_url = api_event.headers.get(DEST_HEADER).unwrap_or_else(|| panic!("Missing destination url"));
    let size = api_event.query_string_parameters.get(SIZE_KEY).unwrap_or_else(|| panic!("Missing size"));

    info!("source_url: {}, dest_url: {}, size: {}", &source_url, &dest_url, &size);
    let result = handle_request(
        &config,
        source_url.to_string(),
        dest_url.to_string(),
        size.to_string()
    );

    let response = ApiGatewayProxyResponse {
        status_code: 200,
        headers: HashMap::new(),
        multi_value_headers:  HashMap::new(),
        is_base64_encoded: Option::from(false),
        body: Option::from(result)
    };

   Ok(response)
}

fn handle_request(config: &Config, source_url: String, dest_url: String, size_as_string: String) -> String {
    let size = size_as_string.parse::<f32>().unwrap();

    let mut source_response = reqwest::get(source_url.as_str()).expect("Failed to download source image");
    let mut source_image_buffer= Vec::new();
    let source_size = source_response.read_to_end(&mut source_image_buffer).unwrap();
    let img = image::load_from_memory(&source_image_buffer)
        .ok()
        .expect("Opening image failed");


    let resized_image_buffer = resize_image(&img, &size).expect("Could not resize image");

    let client = reqwest::Client::new();
    let response = client.put(dest_url.as_str()).body(resized_image_buffer).send();

    if response.is_ok() {
        return "OK".to_string();
    } else {
        panic!("Failed to upload to destination");
    }
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
