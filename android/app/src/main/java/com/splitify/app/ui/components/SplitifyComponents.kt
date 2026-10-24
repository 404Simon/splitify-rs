package com.splitify.app.ui.components

import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.lerp
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.splitify.app.ui.format.formatMoney
import com.splitify.app.ui.format.initialOf
import com.splitify.app.ui.theme.splitify

/** Rounded-square Splitify mark: a two-tone coin that reads as "split". */
@Composable
fun BrandMark(
    modifier: Modifier = Modifier,
    size: Dp = 64.dp,
) {
    val gradient = MaterialTheme.splitify.heroGradient
    Box(
        modifier = modifier
            .size(size)
            .clip(RoundedCornerShape(size * 0.32f))
            .background(Brush.linearGradient(gradient)),
        contentAlignment = Alignment.Center,
    ) {
        Canvas(modifier = Modifier.size(size * 0.54f)) {
            val d = this.size.minDimension
            drawArc(
                color = Color.White,
                startAngle = 92f,
                sweepAngle = 176f,
                useCenter = true,
                topLeft = Offset.Zero,
                size = Size(d, d),
            )
            drawArc(
                color = Color.White.copy(alpha = 0.55f),
                startAngle = 272f,
                sweepAngle = 176f,
                useCenter = true,
                topLeft = Offset.Zero,
                size = Size(d, d),
            )
        }
    }
}

/**
 * Circular initials avatar with a deterministic gradient derived from [seed],
 * so the same person keeps the same color everywhere in the app.
 */
@Composable
fun Avatar(
    name: String?,
    seed: Long,
    modifier: Modifier = Modifier,
    size: Dp = 44.dp,
) {
    val palette = MaterialTheme.splitify.avatarPalette
    val base = palette[paletteIndex(seed, palette.size)]
    val brush = Brush.linearGradient(
        listOf(base, lerp(base, Color.Black, 0.28f)),
    )
    Box(
        modifier = modifier
            .size(size)
            .clip(CircleShape)
            .background(brush),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = initialOf(name),
            color = Color.White,
            fontWeight = FontWeight.SemiBold,
            fontSize = (size.value * 0.38f).sp,
            letterSpacing = 0.sp,
        )
    }
}

/** Overlapping avatar row with an optional `+N` counter. */
@Composable
fun AvatarStack(
    people: List<Pair<Long, String>>,
    modifier: Modifier = Modifier,
    size: Dp = 28.dp,
    max: Int = 4,
    ringColor: Color = MaterialTheme.colorScheme.surface,
) {
    if (people.isEmpty()) return
    val shown = people.take(max)
    val extra = people.size - shown.size
    val slots = shown.size + if (extra > 0) 1 else 0
    val step = size * 0.68f
    val totalWidth = size + step * (slots - 1)

    Box(modifier = modifier.width(totalWidth).height(size)) {
        shown.forEachIndexed { index, (id, name) ->
            Avatar(
                name = name,
                seed = id,
                size = size,
                modifier = Modifier
                    .offset(x = step * index)
                    .border(1.5.dp, ringColor, CircleShape),
            )
        }
        if (extra > 0) {
            Box(
                modifier = Modifier
                    .offset(x = step * shown.size)
                    .size(size)
                    .clip(CircleShape)
                    .background(MaterialTheme.colorScheme.surfaceContainerHighest)
                    .border(1.5.dp, ringColor, CircleShape),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = "+$extra",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

/** Currency text for any server decimal string — always rendered in euros. */
@Composable
fun MoneyText(
    amount: String?,
    modifier: Modifier = Modifier,
    style: TextStyle = MaterialTheme.typography.titleMedium,
    color: Color = MaterialTheme.colorScheme.onSurface,
    showPlus: Boolean = false,
    isNegative: Boolean = false,
) {
    val prefix = when {
        isNegative -> "\u2212"
        showPlus -> "+"
        else -> ""
    }
    Text(
        text = prefix + formatMoney(amount),
        modifier = modifier,
        style = style,
        color = color,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
    )
}

/** Card surface used across the app: white, softly rounded, hairline border. */
@Composable
fun SplitifyCard(
    modifier: Modifier = Modifier,
    onClick: (() -> Unit)? = null,
    fillWidth: Boolean = true,
    contentPadding: PaddingValues = PaddingValues(16.dp),
    content: @Composable ColumnScope.() -> Unit,
) {
    val shape = MaterialTheme.shapes.large
    val background = MaterialTheme.splitify.card
    val border = BorderStroke(1.dp, MaterialTheme.splitify.cardBorder)
    val cardModifier = if (fillWidth) modifier.fillMaxWidth() else modifier

    if (onClick == null) {
        Surface(
            modifier = cardModifier,
            shape = shape,
            color = background,
            contentColor = MaterialTheme.colorScheme.onSurface,
            border = border,
        ) {
            Column(modifier = Modifier.padding(contentPadding), content = content)
        }
    } else {
        Surface(
            onClick = onClick,
            modifier = cardModifier,
            shape = shape,
            color = background,
            contentColor = MaterialTheme.colorScheme.onSurface,
            border = border,
        ) {
            Column(modifier = Modifier.padding(contentPadding), content = content)
        }
    }
}

/** Tinted rounded square holding a single icon; the app's visual accent unit. */
@Composable
fun IconBubble(
    icon: ImageVector,
    modifier: Modifier = Modifier,
    size: Dp = 44.dp,
    iconSize: Dp = 22.dp,
    containerColor: Color = MaterialTheme.colorScheme.primaryContainer,
    contentColor: Color = MaterialTheme.colorScheme.onPrimaryContainer,
    shape: RoundedCornerShape = RoundedCornerShape(size * 0.32f),
) {
    Box(
        modifier = modifier
            .size(size)
            .clip(shape)
            .background(containerColor),
        contentAlignment = Alignment.Center,
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = contentColor,
            modifier = Modifier.size(iconSize),
        )
    }
}

/** Small label pill used for statuses and roles. */
@Composable
fun Pill(
    text: String,
    modifier: Modifier = Modifier,
    containerColor: Color = MaterialTheme.colorScheme.secondaryContainer,
    contentColor: Color = MaterialTheme.colorScheme.onSecondaryContainer,
) {
    Surface(
        modifier = modifier,
        shape = CircleShape,
        color = containerColor,
        contentColor = contentColor,
    ) {
        Text(
            text = text,
            modifier = Modifier.padding(horizontal = 10.dp, vertical = 4.dp),
            style = MaterialTheme.typography.labelSmall,
            maxLines = 1,
        )
    }
}

/** Consistent empty state: tinted icon, title, explanation and a pill CTA. */
@Composable
fun EmptyState(
    icon: ImageVector,
    title: String,
    message: String,
    modifier: Modifier = Modifier,
    accent: Color = MaterialTheme.colorScheme.primary,
    accentContainer: Color = MaterialTheme.colorScheme.primaryContainer,
    actionLabel: String? = null,
    onAction: (() -> Unit)? = null,
) {
    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 32.dp, vertical = 24.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Box(
            modifier = Modifier
                .size(88.dp)
                .clip(CircleShape)
                .background(
                    Brush.radialGradient(
                        listOf(accentContainer, accentContainer.copy(alpha = 0.35f)),
                    ),
                ),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                tint = accent,
                modifier = Modifier.size(38.dp),
            )
        }
        Spacer(Modifier.height(20.dp))
        Text(
            text = title,
            style = MaterialTheme.typography.titleLarge,
            textAlign = TextAlign.Center,
        )
        Spacer(Modifier.height(6.dp))
        Text(
            text = message,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
        )
        if (actionLabel != null && onAction != null) {
            Spacer(Modifier.height(20.dp))
            Button(
                onClick = onAction,
                shape = CircleShape,
                contentPadding = PaddingValues(horizontal = 24.dp, vertical = 12.dp),
            ) {
                Text(actionLabel)
            }
        }
    }
}

/** Section label with an optional trailing action. */
@Composable
fun SectionHeader(
    title: String,
    modifier: Modifier = Modifier,
    subtitle: String? = null,
    trailing: (@Composable () -> Unit)? = null,
) {
    Row(
        modifier = modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(text = title, style = MaterialTheme.typography.titleLarge)
            if (subtitle != null) {
                Text(
                    text = subtitle,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
        trailing?.invoke()
    }
}

/** Tab descriptor for [PillTabRow]. */
data class PillTab(
    val label: String,
    val icon: ImageVector,
)

/**
 * Scrollable pill tabs. Replaces Material's stock TabRow, which crams eight
 * destinations into one dense strip.
 */
@Composable
fun PillTabRow(
    tabs: List<PillTab>,
    selectedIndex: Int,
    onSelected: (Int) -> Unit,
    modifier: Modifier = Modifier,
    contentPadding: PaddingValues = PaddingValues(horizontal = 16.dp, vertical = 10.dp),
) {
    LazyRow(
        modifier = modifier.fillMaxWidth(),
        contentPadding = contentPadding,
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        itemsIndexed(tabs) { index, tab ->
            val selected = index == selectedIndex
            val container by animateColorAsState(
                targetValue = if (selected) {
                    MaterialTheme.colorScheme.primary
                } else {
                    MaterialTheme.colorScheme.surfaceContainerHigh
                },
                label = "pillContainer",
            )
            val contentColor by animateColorAsState(
                targetValue = if (selected) {
                    MaterialTheme.colorScheme.onPrimary
                } else {
                    MaterialTheme.colorScheme.onSurfaceVariant
                },
                label = "pillContent",
            )
            Surface(
                onClick = { onSelected(index) },
                shape = CircleShape,
                color = container,
                contentColor = contentColor,
            ) {
                Row(
                    modifier = Modifier.padding(horizontal = 14.dp, vertical = 9.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(7.dp),
                ) {
                    Icon(
                        imageVector = tab.icon,
                        contentDescription = null,
                        modifier = Modifier.size(17.dp),
                    )
                    Text(text = tab.label, style = MaterialTheme.typography.labelLarge)
                }
            }
        }
    }
}

private fun paletteIndex(seed: Long, size: Int): Int {
    if (size <= 0) return 0
    val mod = seed % size
    return ((mod + size) % size).toInt()
}
