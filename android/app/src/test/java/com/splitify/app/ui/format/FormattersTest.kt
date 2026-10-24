package com.splitify.app.ui.format

import java.util.Locale
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class FormattersTest {

    @Test
    fun `money is always rendered in euros`() {
        val previous = Locale.getDefault()
        try {
            Locale.setDefault(Locale.US)
            val formatted = formatMoney("10.50")
            assertTrue("expected euro symbol in $formatted", formatted.contains("€"))
            assertTrue("expected amount in $formatted", formatted.contains("10.50"))
            assertTrue("expected two decimals in ${formatMoney("7")}", formatMoney("7").contains("7.00"))
        } finally {
            Locale.setDefault(previous)
        }
    }

    @Test
    fun `money keeps malformed values visible`() {
        assertEquals("—", formatMoney(null))
        assertEquals("not-a-number", formatMoney("not-a-number"))
    }

    @Test
    fun `even split rounds half up`() {
        assertEquals("3.33", splitEvenly("10.00", 3))
        assertEquals("2.50", splitEvenly("10", 4))
        assertNull(splitEvenly("nope", 2))
        assertNull(splitEvenly("10.00", 0))
    }
}
