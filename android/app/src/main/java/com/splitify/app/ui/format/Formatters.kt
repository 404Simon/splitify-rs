package com.splitify.app.ui.format

import java.math.BigDecimal
import java.text.NumberFormat
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import java.time.temporal.ChronoUnit
import java.util.Currency
import java.util.Locale

private val Euro: Currency = Currency.getInstance("EUR")

/**
 * Formats a server-side decimal string (e.g. `"12.50"`) as euro currency.
 *
 * The number is parsed as a [BigDecimal] so no floating point rounding is
 * introduced on the client, and the device locale decides the separators —
 * `12,50 €` in Germany, `€12.50` in the US. Values that are not numeric are
 * returned unchanged so a malformed server response never hides information.
 */
fun formatMoney(raw: String?, showPlus: Boolean = false): String {
    val trimmed = raw?.trim().orEmpty()
    if (trimmed.isEmpty()) return "—"

    val amount = trimmed.toBigDecimalOrNull() ?: return trimmed
    val formatter = NumberFormat.getCurrencyInstance(Locale.getDefault()).apply {
        currency = Euro
        minimumFractionDigits = 2
        maximumFractionDigits = 2
    }
    val formatted = formatter.format(amount)
    return if (showPlus && amount > BigDecimal.ZERO) "+$formatted" else formatted
}

/** Parses an ISO-ish timestamp into a local date, ignoring time and zone. */
private fun parseDate(raw: String?): LocalDate? {
    val value = raw?.trim().orEmpty()
    if (value.length < 10) return null
    return runCatching { LocalDate.parse(value.take(10)) }.getOrNull()
}

/** `Today`, `Yesterday`, `4 days ago`, or a short calendar date. */
fun relativeDate(raw: String?): String? {
    val date = parseDate(raw) ?: return null
    val days = ChronoUnit.DAYS.between(date, LocalDate.now())
    return when {
        days == 0L -> "Today"
        days == 1L -> "Yesterday"
        days in 2L..6L -> "$days days ago"
        days == -1L -> "Tomorrow"
        days in -6L..-2L -> "In ${-days} days"
        else -> date.format(DateTimeFormatter.ofPattern("MMM d, yyyy", Locale.getDefault()))
    }
}

/** Short calendar date such as `Oct 24, 2026`. */
fun formatDate(raw: String?): String? {
    val date = parseDate(raw) ?: return null
    return date.format(DateTimeFormatter.ofPattern("MMM d, yyyy", Locale.getDefault()))
}

/** `Oct 24` — used where the year is redundant. */
fun formatShortDate(raw: String?): String? {
    val date = parseDate(raw) ?: return null
    return date.format(DateTimeFormatter.ofPattern("MMM d", Locale.getDefault()))
}

/** `14:05` from an ISO timestamp; null when it cannot be parsed. */
fun formatTime(raw: String?): String? {
    val value = raw?.trim().orEmpty()
    val timePart = value.substringAfter('T', "").takeIf { it.length >= 5 }
        ?: value.substringAfter(' ', "").takeIf { it.length >= 5 }
        ?: return null
    return timePart.take(5)
}

/** Uppercase initial used by avatars; falls back to a neutral glyph. */
fun initialOf(name: String?): String =
    name?.trim()?.take(1)?.uppercase(Locale.getDefault()).orEmpty().ifBlank { "?" }

/**
 * Even split preview for a form field: `"10.00"` over 4 people → `"2.50"`.
 * Returns null when the amount is not a valid number or nobody is selected.
 */
fun splitEvenly(raw: String?, people: Int): String? {
    if (people <= 0) return null
    val amount = raw?.trim()?.toBigDecimalOrNull() ?: return null
    return amount.divide(BigDecimal(people), 2, java.math.RoundingMode.HALF_UP).toPlainString()
}
