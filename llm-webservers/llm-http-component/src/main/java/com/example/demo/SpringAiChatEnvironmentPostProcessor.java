package com.example.demo;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.env.EnvironmentPostProcessor;
import org.springframework.core.env.ConfigurableEnvironment;
import org.springframework.core.env.MapPropertySource;

import java.util.HashMap;

/**
 * @author bingdong.yang@salesforce-china.com
 */

public class SpringAiChatEnvironmentPostProcessor implements EnvironmentPostProcessor {

    @Override
    public void postProcessEnvironment(ConfigurableEnvironment environment, SpringApplication application) {
        // disable the spring ai model auto config to support multitenant
        HashMap<String, Object> map = new HashMap<>();
        map.put("spring.ai.chat.client.enabled", false);
        map.put("spring.ai.openai.chat.enabled", false);
        map.put("spring.ai.openai.embedding.enabled", false);
        map.put("spring.ai.openai.image.enabled", false);
        map.put("spring.ai.openai.audio.transcription.enabled", false);
        map.put("spring.ai.openai.audio.speech.enabled", false);

        // OpenAiAutoConfiguration#openAiModerationClient 并没有提供条件判断, api-key 需要有一个值让它不报错.
        map.put("spring.ai.openai.api-key", "dummy");
        MapPropertySource source = new MapPropertySource("yingwuChat", map);
        environment.getPropertySources().addLast(source);
    }
}
