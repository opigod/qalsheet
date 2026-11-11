package com.qalsheet.convert

import com.qalsheet.calc.CalculatorEngine

/**
 * A unit is converted to the category's base unit with:
 *
 *     base = (value + offset) * factor
 *
 * That single affine form covers lengths/masses (offset 0) and temperatures.
 */
data class MeasureUnit(
    val symbol: String,
    val name: String,
    val factor: Double,
    val offset: Double = 0.0,
) {
    fun toBase(value: Double): Double = (value + offset) * factor

    fun fromBase(value: Double): Double = value / factor - offset
}

data class UnitCategory(
    val label: String,
    val units: List<MeasureUnit>,
    val defaultFrom: String,
    val defaultTo: String,
) {
    fun unit(symbol: String): MeasureUnit = units.first { it.symbol == symbol }

    fun indexOf(symbol: String): Int = units.indexOfFirst { it.symbol == symbol }.coerceAtLeast(0)

    val defaultFromIndex: Int get() = indexOf(defaultFrom)
    val defaultToIndex: Int get() = indexOf(defaultTo)
}

object Units {

    val length = UnitCategory(
        label = "Length",
        units = listOf(
            MeasureUnit("mm", "Millimeter", 0.001),
            MeasureUnit("cm", "Centimeter", 0.01),
            MeasureUnit("dm", "Decimeter", 0.1),
            MeasureUnit("m", "Meter", 1.0),
            MeasureUnit("km", "Kilometer", 1_000.0),
            MeasureUnit("in", "Inch", 0.0254),
            MeasureUnit("ft", "Foot", 0.3048),
            MeasureUnit("yd", "Yard", 0.9144),
            MeasureUnit("mi", "Mile", 1_609.344),
        ),
        defaultFrom = "cm",
        defaultTo = "mm",
    )

    val mass = UnitCategory(
        label = "Mass",
        units = listOf(
            MeasureUnit("mg", "Milligram", 0.001),
            MeasureUnit("g", "Gram", 1.0),
            MeasureUnit("kg", "Kilogram", 1_000.0),
            MeasureUnit("t", "Tonne", 1_000_000.0),
            MeasureUnit("oz", "Ounce", 28.349523125),
            MeasureUnit("lb", "Pound", 453.59237),
            MeasureUnit("st", "Stone", 6_350.29318),
        ),
        defaultFrom = "kg",
        defaultTo = "g",
    )

    val temperature = UnitCategory(
        label = "Temp",
        units = listOf(
            MeasureUnit("°C", "Celsius", 1.0, 0.0),
            MeasureUnit("°F", "Fahrenheit", 5.0 / 9.0, -32.0),
            MeasureUnit("K", "Kelvin", 1.0, -273.15),
            MeasureUnit("°R", "Rankine", 5.0 / 9.0, -491.67),
        ),
        defaultFrom = "°C",
        defaultTo = "°F",
    )

    val volume = UnitCategory(
        label = "Volume",
        units = listOf(
            MeasureUnit("ml", "Milliliter", 0.001),
            MeasureUnit("cl", "Centiliter", 0.01),
            MeasureUnit("dl", "Deciliter", 0.1),
            MeasureUnit("l", "Liter", 1.0),
            MeasureUnit("m³", "Cubic meter", 1_000.0),
            MeasureUnit("tsp", "Teaspoon (US)", 0.00492892159375),
            MeasureUnit("tbsp", "Tablespoon (US)", 0.01478676478125),
            MeasureUnit("fl oz", "Fluid ounce (US)", 0.0295735295625),
            MeasureUnit("cup", "Cup (US)", 0.2365882365),
            MeasureUnit("pint", "Pint (US)", 0.473176473),
            MeasureUnit("gal", "Gallon (US)", 3.785411784),
        ),
        defaultFrom = "l",
        defaultTo = "ml",
    )

    val area = UnitCategory(
        label = "Area",
        units = listOf(
            MeasureUnit("mm²", "Square millimeter", 0.000001),
            MeasureUnit("cm²", "Square centimeter", 0.0001),
            MeasureUnit("dm²", "Square decimeter", 0.01),
            MeasureUnit("m²", "Square meter", 1.0),
            MeasureUnit("km²", "Square kilometer", 1_000_000.0),
            MeasureUnit("ha", "Hectare", 10_000.0),
            MeasureUnit("in²", "Square inch", 0.00064516),
            MeasureUnit("ft²", "Square foot", 0.09290304),
            MeasureUnit("yd²", "Square yard", 0.83612736),
            MeasureUnit("acre", "Acre", 4_046.8564224),
        ),
        defaultFrom = "m²",
        defaultTo = "cm²",
    )

    val speed = UnitCategory(
        label = "Speed",
        units = listOf(
            MeasureUnit("m/s", "Meter / second", 1.0),
            MeasureUnit("km/h", "Kilometer / hour", 1.0 / 3.6),
            MeasureUnit("mph", "Mile / hour", 0.44704),
            MeasureUnit("kn", "Knot", 0.5144444444444445),
            MeasureUnit("ft/s", "Foot / second", 0.3048),
        ),
        defaultFrom = "km/h",
        defaultTo = "m/s",
    )

    val time = UnitCategory(
        label = "Time",
        units = listOf(
            MeasureUnit("ms", "Millisecond", 0.001),
            MeasureUnit("s", "Second", 1.0),
            MeasureUnit("min", "Minute", 60.0),
            MeasureUnit("h", "Hour", 3_600.0),
            MeasureUnit("day", "Day", 86_400.0),
            MeasureUnit("wk", "Week", 604_800.0),
            MeasureUnit("yr", "Year (365.25 d)", 31_557_600.0),
        ),
        defaultFrom = "h",
        defaultTo = "min",
    )

    val data = UnitCategory(
        label = "Data",
        units = listOf(
            MeasureUnit("bit", "Bit", 0.125),
            MeasureUnit("B", "Byte", 1.0),
            MeasureUnit("kB", "Kilobyte (1000 B)", 1_000.0),
            MeasureUnit("MB", "Megabyte (1000² B)", 1_000_000.0),
            MeasureUnit("GB", "Gigabyte (1000³ B)", 1_000_000_000.0),
            MeasureUnit("TB", "Terabyte (1000⁴ B)", 1_000_000_000_000.0),
            MeasureUnit("KiB", "Kibibyte (1024 B)", 1_024.0),
            MeasureUnit("MiB", "Mebibyte (1024² B)", 1_048_576.0),
            MeasureUnit("GiB", "Gibibyte (1024³ B)", 1_073_741_824.0),
            MeasureUnit("TiB", "Tebibyte (1024⁴ B)", 1_099_511_627_776.0),
        ),
        defaultFrom = "MB",
        defaultTo = "GB",
    )

    val categories = listOf(length, mass, temperature, volume, area, speed, time, data)

    fun convert(value: Double, from: MeasureUnit, to: MeasureUnit): Double = to.fromBase(from.toBase(value))

    fun format(value: Double): String = CalculatorEngine.format(value, grouping = true)

    fun formatEditable(value: Double): String = CalculatorEngine.formatEditable(value)

    fun formatReference(from: MeasureUnit, to: MeasureUnit): String {
        val amount = convert(1.0, from, to)
        return "1 ${from.symbol} = ${format(amount)} ${to.symbol}"
    }
}
