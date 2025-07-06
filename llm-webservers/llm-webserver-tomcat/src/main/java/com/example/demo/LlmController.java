package com.example.demo;

import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RestController;

/**
 * @author yangbingdong1994@gmail.com
 */
@RestController
public class LlmController {

    private final LlmService llmService;

    public LlmController(LlmService llmService) {
        this.llmService = llmService;
    }

    @PostMapping("/compatible-mode/v1/chat/completions")
    public String chatCompletion() {
        llmService.think();
        return "success";
    }
}
