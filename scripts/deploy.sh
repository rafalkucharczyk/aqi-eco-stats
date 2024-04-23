#!/bin/bash -x

PROJECT_NAME=aqi-eco-stats
TYPE=dev
STACK_NAME=$PROJECT_NAME-$TYPE-stack
ACTION=create

cd backend

cargo lambda build --release --arm64 --output-format zip
CODE_HASH=$(unzip -p target/lambda/test_app/bootstrap.zip bootstrap | openssl dgst -md5 | cut -f2 -d' ')
TARGET_FILE=fetch-and-store/bootstrap-$CODE_HASH.zip
S3_PATH=s3://$PROJECT_NAME-$TYPE-lambda-bucket/$TARGET_FILE

if ! aws s3 ls $S3_PATH; then
     aws s3 cp ./target/lambda/lambda_fetch_and_store/bootstrap.zip $S3_PATH
fi

if aws cloudformation describe-stacks --stack-name $STACK_NAME --no-cli-pager; then
    ACTION=update
fi

aws cloudformation $ACTION-stack \
    --stack-name $STACK_NAME \
    --template-body file://../cloudformation/backend.yml \
    --parameters ParameterKey=EnvironmentType,ParameterValue=$TYPE  \
    --parameters ParameterKey=FetchAndStoreLambdaFile,ParameterValue=$TARGET_FILE  \
    --capabilities CAPABILITY_IAM \
    --disable-rollback \
    --no-cli-pager

if [[ $? -eq 0 ]]; then
    aws cloudformation wait stack-$ACTION-complete --stack-name $STACK_NAME --no-cli-pager
fi
