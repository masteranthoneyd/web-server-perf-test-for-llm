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
        for (int i = 0; i < 500; i++) {
            Thread.startVirtualThread(new Runnable() {
                @Override
                public void run() {
                    callLlmMApiock();
                }
            });
        }
    }

    private void callLlmMApiock() {
        ResponseEntity<OpenAiApi.ChatCompletion> entity = builder.build().post()
                .uri("http://10.5.148.136:8080/compatible-mode/v1/chat/completions")
//                .body(chatRequest)
                .retrieve()
                .toEntity(OpenAiApi.ChatCompletion.class);
        log.info("result: {}", entity.getBody());
    }
}
