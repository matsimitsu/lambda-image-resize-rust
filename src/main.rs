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

use image::{GenericImageView, ImageError, JPEG};
use rayon::prelude::*;
use s3::bucket::Bucket;
use s3::credentials::Credentials;
use s3::region::Region;

mod config;

use aws_lambda_events::event::s3::{S3Event, S3EventRecord};
use config::Config;
use lambda::error::HandlerError;
use serde_json::Value;
use std::error::Error;

fn main() -> Result<(), Box<Error>> {
    simple_logger::init_with_level(log::Level::Info)?;

    lambda!(handle_event);

    Ok(())
}

fn handle_event(event: Value, ctx: lambda::Context) -> Result<(), HandlerError> {
    let config = Config::new();

    let s3_event: S3Event =
        serde_json::from_value(event).map_err(|e| ctx.new_error(e.to_string().as_str()))?;

    for record in s3_event.records {
        handle_record(&config, record);
    }
    Ok(())
}

fn handle_record(config: &Config, record: S3EventRecord) {
    let credentials = Credentials::default();
    let region: Region = record
        .aws_region
        .expect("Could not get region from record")
        .parse()
        .expect("Could not parse region from record");
    let bucket = Bucket::new(
        &record
            .s3
            .bucket
            .name
            .expect("Could not get bucket name from record"),
        region,
        credentials,
    );
    let source = record
        .s3
        .object
        .key
        .expect("Could not get key from object record");
    info!("Fetching: {}, config: {:?}", &source, &config);

    /* Make sure we don't process files twice */
    for size in &config.sizes {
        let to_match = format!("-{}.jpg", size);
        if source.ends_with(&to_match) {
            warn!(
                "Source: '{}' ends with: '{}'. Skipping.",
                &source,
                &to_match
            );
            return;
        }
    }

    let (data, _) = bucket
        .get(&source)
        .expect(&format!("Could not get object: {}", &source));

    let img = image::load_from_memory(&data)
        .ok()
        .expect("Opening image failed");

    let _: Vec<_> = config
        .sizes
        .par_iter()
        .map(|size| {
            let buffer = resize_image(&img, &size).expect("Could not resize image");

            let mut target = source.clone();
            for (rep_key, rep_val) in &config.replacements {
                target = target.replace(rep_key, rep_val);
            }
            target = target.replace(".jpg", &format!("-{}.jpg", size));
            let (_, code) = bucket
                .put(&target, &buffer, "image/jpeg")
                .expect(&format!("Could not upload object to :{}", &target));
            info!("Uploaded: {} with: {}", &target, &code);
        })
        .collect();
}

fn resize_image(img: &image::DynamicImage, new_w: &f32) -> Result<Vec<u8>, ImageError> {
    let mut result: Vec<u8> = Vec::new();

    let old_w = img.width() as f32;
    let old_h = img.height() as f32;
    let ratio = new_w / old_w;
    let new_h = (old_h * ratio).floor();

    let scaled = img.resize(*new_w as u32, new_h as u32, image::FilterType::Lanczos3);
    scaled.write_to(&mut result, JPEG)?;

    Ok(result)
}
