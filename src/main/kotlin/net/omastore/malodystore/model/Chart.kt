package net.omastore.malodystore.model

data class Chart(
    val cid: Int,
    val uid: Int,
    val creator: String,
    val version: String,
    val level: Int,
    val length: Int,
    val type: Int,
    val size: Int,
    val mode: Int,
)
