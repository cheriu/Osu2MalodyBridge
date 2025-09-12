package net.omastore.malodystore.model

/**
 *
 * sid: song id, unique identifier
 *
 * cover: full cover url
 *
 * length: the length of the song, in seconds
 *
 * bpm: song bpm, float point
 *
 * mode: The bitmask value of the chart type included in the song. For example, the song contains both key and catch modes, and the bitmask value is (1 << 0) | (1 << 3) = 9
 *
 * time: Last update time of the song
 */
data class Song(
    val sid: Int,
    val cover: String,
    val length: Int,
    val bpm: Float,
    val title: String,
    val artist: String,
    val mode: Int,
    val time: Long,
)
