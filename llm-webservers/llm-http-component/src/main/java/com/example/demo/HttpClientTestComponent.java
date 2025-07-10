package com.example.demo;

import lombok.extern.slf4j.Slf4j;
import org.springframework.ai.openai.api.OpenAiApi;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.http.ResponseEntity;
import org.springframework.stereotype.Component;
import org.springframework.web.client.RestClient;

import java.net.http.HttpClient;

/**
 * @author bingdong.yang@salesforce-china.com
 */
@Slf4j
@Component
public class HttpClientTestComponent {

    @Autowired
    private RestClient.Builder builder;

    public void test() {
//        callLlmRealApi();
        for (int i = 0; i < 5000; i++) {
            Thread.startVirtualThread(new Runnable() {
                @Override
                public void run() {
                    callLlmMockApi();
//                    callLlmRealApi();
                }
            });
        }
//        callLlmRealApi();
//        callLlmMockApi();
    }

    private void callLlmMockApi() {
        ResponseEntity<OpenAiApi.ChatCompletion> entity = builder.build().post()
                .uri("http://10.5.148.136:8080/compatible-mode/v1/chat/completions")
//                .body(chatRequest)
                .retrieve()
                .toEntity(OpenAiApi.ChatCompletion.class);
        log.info("result: {}", entity.getBody());
    }

    private void callLlmRealApi() {
        String body = """
                {
                    "model": "qwen-plus",
                    "messages": [
                        {
                            "role": "system",
                            "content": "You are a helpful assistant."
                        },
                        {
                            "role": "user",
                            "content": "给出一份制作意大利面的操作流程"
                        }
                    ]
                }
                """;
        ResponseEntity<OpenAiApi.ChatCompletion> entity = builder.build().post()
                .uri("https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions")
                .body(body)
                .header("Authorization", "Bearer ***")
                .header("Content-Type", "application/json")
                .retrieve()
                .toEntity(OpenAiApi.ChatCompletion.class);
        log.info("result: {}", entity.getBody());
    }
}
