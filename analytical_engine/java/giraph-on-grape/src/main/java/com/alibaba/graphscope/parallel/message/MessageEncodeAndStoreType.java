/*
 * Copyright 2021 Alibaba Group Holding Limited.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *  	http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
package com.alibaba.graphscope.parallel.message;

/**
 * There are two types of message-stores currently pointer based, and default byte-array based
 */
public enum MessageEncodeAndStoreType {
    SIMPLE_MESSAGE_STORE(false);

    /**
     * Can use one message to many ids encoding?
     */
    private final boolean oneMessageToManyIdsEncoding;

    /**
     * Constructor
     *
     * @param oneMessageToManyIdsEncoding use one message to many ids encoding
     */
    MessageEncodeAndStoreType(boolean oneMessageToManyIdsEncoding) {
        this.oneMessageToManyIdsEncoding = oneMessageToManyIdsEncoding;
    }

    /**
     * True if one message to many ids encoding is set
     *
     * @return return oneMessageToManyIdsEncoding
     */
    public boolean useOneMessageToManyIdsEncoding() {
        return oneMessageToManyIdsEncoding;
    }
}