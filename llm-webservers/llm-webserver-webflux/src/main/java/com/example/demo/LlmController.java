package com.example.demo;

import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RestController;
import reactor.core.publisher.Mono;

/**
 * @author yangbingdong1994@gmail.com
 */
@RestController
public class LlmController {
    private static final String resp = """
            {
                "choices": [
                    {
                        "message": {
                            "role": "assistant",
                            "content": "我是阿里云开发的一款超大规模语言模型，我叫通义千问。"
                        },
                        "finish_reason": "stop",
                        "index": 0,
                        "logprobs": null
                    }
                ],
                "object": "chat.completion",
                "usage": {
                    "prompt_tokens": 3019,
                    "completion_tokens": 104,
                    "total_tokens": 3123,
                    "prompt_tokens_details": {
                        "cached_tokens": 2048
                    }
                },
                "created": 1735120033,
                "system_fingerprint": null,
                "model": "qwen-plus",
                "id": "chatcmpl-6ada9ed2-7f33-9de2-8bb0-78bd4035025a"
            }
            """;

    private final LlmService llmService;

    public LlmController(LlmService llmService) {
        this.llmService = llmService;
    }

    @PostMapping("/compatible-mode/v1/chat/completions")
    public Mono<String> chatCompletion() {
        return llmService.think()
                .then(Mono.just(resp));
    }
}
