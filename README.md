# Lambda Image Resize

A simple image resize lambda function, written in Rust.

This binary responds to Amazon S3 events and triggers a resize on the uploaded image with the sizes specified. Right now you can only resize on the width of an image.

## Configure

This binary relies on two env vars:

* `SIZES`, an array if sizes (`export SIZES=200,300,400`)
* `REPLACEMENTS`, an array of replacements for the path (`export REPLACEMENTS="original:resized"`)

## Compile

Use [Lambda-Rust docker image](https://hub.docker.com/r/softprops/lambda-rust/) to compile this binary. With Docker running run the following command to build a release.

```
make build
```

You can find the (zipped) bootstrap ready for upload to AWS Lambda in `target/lambda/release/bootstrap.zip`
