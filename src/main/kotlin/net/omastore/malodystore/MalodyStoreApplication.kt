package net.omastore.malodystore

import org.springframework.boot.autoconfigure.SpringBootApplication
import org.springframework.boot.runApplication

@SpringBootApplication
class MalodyStoreApplication

fun main(args: Array<String>) {
    runApplication<MalodyStoreApplication>(*args)
}
