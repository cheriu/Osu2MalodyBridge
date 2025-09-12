package net.omastore.malodystore.service.impl

import net.omastore.malodystore.config.MalodyServerProperties
import net.omastore.malodystore.model.ServerInfoResponse
import net.omastore.malodystore.service.BasicInformation
import org.springframework.stereotype.Service

@Service
class BasicInformationImpl(
    private val malodyServerProperties: MalodyServerProperties,
) : BasicInformation {
    override fun info(): ServerInfoResponse =
        ServerInfoResponse(
            code = 0,
            api = malodyServerProperties.api,
            min = malodyServerProperties.min,
            welcome = malodyServerProperties.welcome,
        )
}
