#!/bin/bash

aws dynamodb create-table --table-name stats --attribute-definitions AttributeName=time,AttributeType=N --key-schema AttributeName=time,KeyType=HASH --billing-mode PAY_PER_REQUEST --endpoint-url http://localhost:8000
