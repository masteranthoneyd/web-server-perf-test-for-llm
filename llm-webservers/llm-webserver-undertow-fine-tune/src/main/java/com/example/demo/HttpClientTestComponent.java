package com.example.demo;

import lombok.extern.slf4j.Slf4j;
import org.springframework.ai.openai.api.OpenAiApi;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.http.ResponseEntity;
import org.springframework.stereotype.Component;
import org.springframework.web.client.RestClient;

/**
 * @author bingdong.yang@salesforce-china.com
 */
@Slf4j
@Component
public class HttpClientTestComponent {

    @Autowired
    private RestClient.Builder builder;

    public void callLlmMockApi() {
        ResponseEntity<OpenAiApi.ChatCompletion> entity = builder.build().post()
                .uri("http://127.0.0.1:8081/compatible-mode/v1/chat/completions")
                .retrieve()
                .toEntity(OpenAiApi.ChatCompletion.class);
        log.info("result: {}", entity.getBody());
    }

}
