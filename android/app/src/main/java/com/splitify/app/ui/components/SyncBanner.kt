package com.splitify.app.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.splitify.app.data.sync.SyncStatus

/** Whether [SyncBanner] renders anything for the given state. */
fun isSyncBannerVisible(
    status: SyncStatus,
    pendingCount: Int,
    isOnline: Boolean,
): Boolean = !isOnline || status is SyncStatus.Syncing || status is SyncStatus.Error ||
    pendingCount > 0

/** Global sync state strip shown above the navigation host. */
@Composable
fun SyncBanner(
    status: SyncStatus,
    pendingCount: Int,
    isOnline: Boolean,
    onSyncNow: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val presentation = syncBannerPresentation(status, pendingCount, isOnline) ?: return

    Surface(
        modifier = modifier.fillMaxWidth(),
        color = presentation.containerColor,
        contentColor = presentation.contentColor,
    ) {
        Row(
            modifier = Modifier
                .windowInsetsPadding(WindowInsets.statusBars)
                .fillMaxWidth()
                .padding(start = 16.dp, end = 8.dp, top = 4.dp, bottom = 4.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            if (presentation.showProgress) {
                CircularProgressIndicator(
                    modifier = Modifier.size(14.dp),
                    color = presentation.contentColor,
                    strokeWidth = 2.dp,
                )
            }
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = presentation.message,
                    style = MaterialTheme.typography.bodyMedium,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
                presentation.detail?.let { detail ->
                    Text(
                        text = detail,
                        style = MaterialTheme.typography.labelMedium,
                    )
                }
            }
            if (presentation.actionLabel != null) {
                TextButton(
                    onClick = onSyncNow,
                    contentPadding = PaddingValues(horizontal = 8.dp),
                ) {
                    Text(presentation.actionLabel)
                }
            }
        }
    }
}

private data class BannerPresentation(
    val message: String,
    val containerColor: Color,
    val contentColor: Color,
    val detail: String? = null,
    val actionLabel: String? = null,
    val showProgress: Boolean = false,
)

@Composable
private fun syncBannerPresentation(
    status: SyncStatus,
    pendingCount: Int,
    isOnline: Boolean,
): BannerPresentation? {
    val scheme = MaterialTheme.colorScheme
    return when {
        !isOnline -> BannerPresentation(
            message = "Offline — showing saved data",
            detail = changeLabel(pendingCount).takeIf { pendingCount > 0 }?.let { "$it waiting" },
            containerColor = scheme.surfaceVariant,
            contentColor = scheme.onSurfaceVariant,
        )

        status is SyncStatus.Error -> BannerPresentation(
            message = status.message,
            containerColor = scheme.errorContainer,
            contentColor = scheme.onErrorContainer,
            actionLabel = "Retry",
        )

        status is SyncStatus.Syncing -> BannerPresentation(
            message = "Syncing…",
            containerColor = scheme.secondaryContainer,
            contentColor = scheme.onSecondaryContainer,
            showProgress = true,
        )

        pendingCount > 0 -> BannerPresentation(
            message = "${changeLabel(pendingCount)} waiting to sync",
            containerColor = scheme.tertiaryContainer,
            contentColor = scheme.onTertiaryContainer,
            actionLabel = "Sync now",
        )

        else -> null
    }
}

private fun changeLabel(count: Int): String =
    if (count == 1) "1 change" else "$count changes"
