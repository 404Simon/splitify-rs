package com.splitify.app.ui.theme

import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Shapes
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.ReadOnlyComposable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp

private val LightColorScheme = lightColorScheme(
    primary = LightPrimary,
    onPrimary = LightOnPrimary,
    primaryContainer = LightPrimaryContainer,
    onPrimaryContainer = LightOnPrimaryContainer,
    inversePrimary = LightInversePrimary,
    secondary = LightSecondary,
    onSecondary = LightOnSecondary,
    secondaryContainer = LightSecondaryContainer,
    onSecondaryContainer = LightOnSecondaryContainer,
    tertiary = LightTertiary,
    onTertiary = LightOnTertiary,
    tertiaryContainer = LightTertiaryContainer,
    onTertiaryContainer = LightOnTertiaryContainer,
    background = LightBackground,
    onBackground = LightOnBackground,
    surface = LightSurface,
    onSurface = LightOnSurface,
    surfaceVariant = LightSurfaceVariant,
    onSurfaceVariant = LightOnSurfaceVariant,
    surfaceTint = LightPrimary,
    surfaceDim = LightSurfaceDim,
    surfaceBright = LightSurfaceBright,
    surfaceContainerLowest = LightSurfaceLowest,
    surfaceContainerLow = LightSurfaceLow,
    surfaceContainer = LightSurfaceContainer,
    surfaceContainerHigh = LightSurfaceHigh,
    surfaceContainerHighest = LightSurfaceHighest,
    inverseSurface = LightInverseSurface,
    inverseOnSurface = LightInverseOnSurface,
    outline = LightOutline,
    outlineVariant = LightOutlineVariant,
    error = ErrorRed,
    onError = OnErrorRed,
    errorContainer = ErrorContainer,
    onErrorContainer = OnErrorContainer,
)

private val DarkColorScheme = darkColorScheme(
    primary = DarkPrimary,
    onPrimary = DarkOnPrimary,
    primaryContainer = DarkPrimaryContainer,
    onPrimaryContainer = DarkOnPrimaryContainer,
    inversePrimary = DarkInversePrimary,
    secondary = DarkSecondary,
    onSecondary = DarkOnSecondary,
    secondaryContainer = DarkSecondaryContainer,
    onSecondaryContainer = DarkOnSecondaryContainer,
    tertiary = DarkTertiary,
    onTertiary = DarkOnTertiary,
    tertiaryContainer = DarkTertiaryContainer,
    onTertiaryContainer = DarkOnTertiaryContainer,
    background = DarkBackground,
    onBackground = DarkOnBackground,
    surface = DarkSurface,
    onSurface = DarkOnSurface,
    surfaceVariant = DarkSurfaceVariant,
    onSurfaceVariant = DarkOnSurfaceVariant,
    surfaceTint = DarkPrimary,
    surfaceDim = DarkSurfaceDim,
    surfaceBright = DarkSurfaceBright,
    surfaceContainerLowest = DarkSurfaceLowest,
    surfaceContainerLow = DarkSurfaceLow,
    surfaceContainer = DarkSurfaceContainer,
    surfaceContainerHigh = DarkSurfaceHigh,
    surfaceContainerHighest = DarkSurfaceHighest,
    inverseSurface = DarkInverseSurface,
    inverseOnSurface = DarkInverseOnSurface,
    outline = DarkOutline,
    outlineVariant = DarkOutlineVariant,
    error = DarkErrorRed,
    onError = DarkOnErrorRed,
    errorContainer = DarkErrorContainer,
    onErrorContainer = DarkOnErrorContainer,
)

/** Brand-wide corner radii. Cards and sheets read as generous and soft. */
val SplitifyShapes = Shapes(
    extraSmall = RoundedCornerShape(8.dp),
    small = RoundedCornerShape(12.dp),
    medium = RoundedCornerShape(16.dp),
    large = RoundedCornerShape(22.dp),
    extraLarge = RoundedCornerShape(28.dp),
)

/**
 * Semantic colors that Material's [androidx.compose.material3.ColorScheme] does
 * not model: money direction, card surfaces and the avatar palette.
 */
@Immutable
data class SplitifyColors(
    val positive: Color,
    val positiveContainer: Color,
    val onPositiveContainer: Color,
    val negative: Color,
    val negativeContainer: Color,
    val onNegativeContainer: Color,
    val warning: Color,
    val warningContainer: Color,
    val onWarningContainer: Color,
    val card: Color,
    val cardBorder: Color,
    val heroGradient: List<Color>,
    val avatarPalette: List<Color>,
)

private val LightSplitifyColors = SplitifyColors(
    positive = Color(0xFF0E7B60),
    positiveContainer = Color(0xFFC6F2E2),
    onPositiveContainer = Color(0xFF00281D),
    negative = Color(0xFFB3261E),
    negativeContainer = Color(0xFFFFDAD5),
    onNegativeContainer = Color(0xFF410E0A),
    warning = Color(0xFF8A5100),
    warningContainer = Color(0xFFFFE0B8),
    onWarningContainer = Color(0xFF2C1600),
    card = Color(0xFFFFFFFF),
    cardBorder = Color(0xFFE2EAE6),
    heroGradient = listOf(Color(0xFF0B5D51), Color(0xFF0E7C6B), Color(0xFF19A98F)),
    avatarPalette = listOf(
        Color(0xFF0F766E),
        Color(0xFF2563EB),
        Color(0xFF7C3AED),
        Color(0xFFDB2777),
        Color(0xFFB45309),
        Color(0xFF059669),
        Color(0xFF0891B2),
        Color(0xFF4F46E5),
    ),
)

private val DarkSplitifyColors = SplitifyColors(
    positive = Color(0xFF6FDDB4),
    positiveContainer = Color(0xFF0B3B2D),
    onPositiveContainer = Color(0xFFB8F5DD),
    negative = Color(0xFFFFB4AB),
    negativeContainer = Color(0xFF4A0F0D),
    onNegativeContainer = Color(0xFFFFDAD5),
    warning = Color(0xFFFFB86B),
    warningContainer = Color(0xFF4A2A00),
    onWarningContainer = Color(0xFFFFDCBE),
    card = Color(0xFF171E1B),
    cardBorder = Color(0xFF2C3733),
    heroGradient = listOf(Color(0xFF0A3F37), Color(0xFF0D5A4E), Color(0xFF0E7160)),
    avatarPalette = listOf(
        Color(0xFF14B8A6),
        Color(0xFF3B82F6),
        Color(0xFF8B5CF6),
        Color(0xFFEC4899),
        Color(0xFFF59E0B),
        Color(0xFF10B981),
        Color(0xFF06B6D4),
        Color(0xFF6366F1),
    ),
)

private val LocalSplitifyColors = staticCompositionLocalOf { LightSplitifyColors }

/** Access Splitify's semantic colors: `MaterialTheme.splitify.positive`. */
val MaterialTheme.splitify: SplitifyColors
    @Composable
    @ReadOnlyComposable
    get() = LocalSplitifyColors.current

@Composable
fun SplitifyTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    /**
     * Material You dynamic color is intentionally off by default: wallpaper
     * tints would wash out the brand and make amounts harder to read. Callers
     * that really want it (e.g. a future settings toggle) can opt in.
     */
    dynamicColor: Boolean = false,
    content: @Composable () -> Unit,
) {
    val colorScheme = when {
        dynamicColor && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {
            val context = LocalContext.current
            if (darkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
        }

        darkTheme -> DarkColorScheme
        else -> LightColorScheme
    }

    val splitifyColors = if (darkTheme) DarkSplitifyColors else LightSplitifyColors

    CompositionLocalProvider(LocalSplitifyColors provides splitifyColors) {
        MaterialTheme(
            colorScheme = colorScheme,
            typography = SplitifyTypography,
            shapes = SplitifyShapes,
            content = content,
        )
    }
}
