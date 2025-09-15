package net.omastore.malodystore.service

import net.omastore.malodystore.model.ServerInfoResponse

/**
 * see definition at https://gitlab.com/mugzone_team/malody_store_api/-/blob/main/README.md
 */

interface BasicInformation {
    /**
     * Purpose: The client send this request to server immediately after player enter the server host. The server host is not available until the request returns successful and the server API version is compatible with the client API version.
     */
    fun info(): ServerInfoResponse
}
