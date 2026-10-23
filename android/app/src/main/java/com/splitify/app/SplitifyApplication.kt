package com.splitify.app

import android.app.Application
import com.splitify.app.di.AppContainer

class SplitifyApplication : Application() {

    lateinit var container: AppContainer
        private set

    override fun onCreate() {
        super.onCreate()
        container = AppContainer(this)
    }
}
