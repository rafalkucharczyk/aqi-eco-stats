#!/bin/bash -x

TYPE=dev
STACK_NAME=app-$TYPE-stack
ACTION=create

cargo lambda build --release --arm64 --output-format zip
aws s3 cp ./target/lambda/test_app/bootstrap.zip s3://aqi-eco-stats-$TYPE-lambda-bucket/fetch-and-store/

if aws cloudformation describe-stacks --stack-name $STACK_NAME --no-cli-pager; then
    ACTION=update
fi

aws cloudformation $ACTION-stack \
    --stack-name $STACK_NAME \
    --template-body file://infra/infra.yml \
    --parameters ParameterKey=EnvironmentType,ParameterValue=$TYPE  \
    --capabilities CAPABILITY_IAM \
    --disable-rollback \
    --no-cli-pager

if [[ $? -eq 0 ]]; then
    aws cloudformation wait stack-$ACTION-complete --stack-name $STACK_NAME --no-cli-pager
fi
