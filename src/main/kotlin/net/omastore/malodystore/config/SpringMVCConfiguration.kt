package net.omastore.malodystore.config

import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.databind.SerializationFeature
import org.springframework.context.annotation.Bean
import org.springframework.context.annotation.Configuration

@Configuration
class SpringMVCConfiguration {
    @Bean
    fun objectMapper(): ObjectMapper =
        ObjectMapper()
            .enable(SerializationFeature.INDENT_OUTPUT)
}
