package com.splitify.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import com.splitify.app.ui.AppRoot
import com.splitify.app.ui.theme.SplitifyTheme

class MainActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        val container = (application as SplitifyApplication).container
        setContent {
            SplitifyTheme {
                AppRoot(container = container)
            }
        }
    }
}
