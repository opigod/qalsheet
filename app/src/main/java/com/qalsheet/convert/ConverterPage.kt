package com.qalsheet.convert

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.wear.compose.foundation.lazy.ScalingLazyColumn
import androidx.wear.compose.foundation.lazy.items
import androidx.wear.compose.foundation.lazy.rememberScalingLazyListState
import com.qalsheet.ui.CircleIconButton
import com.qalsheet.ui.DialogHeader
import com.qalsheet.ui.FittedText
import com.qalsheet.ui.Glyph
import com.qalsheet.ui.GlyphIcon
import com.qalsheet.ui.QalColors
import com.qalsheet.ui.QalDialog
import com.qalsheet.ui.QalPill
import com.qalsheet.ui.QalType

@Composable
fun ConverterPage(
    state: ConverterState,
    modifier: Modifier = Modifier,
) {
    var editing by remember { mutableStateOf(false) }
    var pickerTarget by remember { mutableStateOf<UnitTarget?>(null) }
    val categoryListState = rememberLazyListState()

    LaunchedEffect(state.categoryIndex) {
        categoryListState.animateScrollToItem(state.categoryIndex)
    }

    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(horizontal = 6.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        LazyRow(
            modifier = Modifier
                .width(178.dp)
                .height(24.dp),
            state = categoryListState,
            horizontalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            itemsIndexed(Units.categories) { index, category ->
                QalPill(
                    text = category.label,
                    onClick = { state.selectCategory(index) },
                    selected = state.categoryIndex == index,
                )
            }
        }

        Spacer(Modifier.height(5.dp))

        ConverterCard(
            state = state,
            onEdit = { editing = true },
            onPickFrom = { pickerTarget = UnitTarget.From },
            onPickTo = { pickerTarget = UnitTarget.To },
            onSwap = { state.swap() },
        )

        Spacer(Modifier.height(7.dp))

        BasicText(
            text = Units.formatReference(state.from, state.to),
            modifier = Modifier.fillMaxWidth(),
            style = QalType.Caption.copy(
                color = QalColors.Muted,
                textAlign = TextAlign.Center,
            ),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        BasicText(
            text = "Tap a value to edit",
            modifier = Modifier.fillMaxWidth(),
            style = QalType.Micro.copy(
                color = QalColors.Faint,
                textAlign = TextAlign.Center,
            ),
            maxLines = 1,
        )
    }

    if (editing) {
        AmountDialog(
            state = state,
            onDismiss = { editing = false },
        )
    }

    pickerTarget?.let { target ->
        UnitPickerDialog(
            category = state.category,
            selectedIndex = if (target == UnitTarget.From) state.fromIndex else state.toIndex,
            title = if (target == UnitTarget.From) "From unit" else "To unit",
            onSelect = { index ->
                state.selectUnit(target, index)
                pickerTarget = null
            },
            onDismiss = { pickerTarget = null },
        )
    }
}

@Composable
private fun ConverterCard(
    state: ConverterState,
    onEdit: () -> Unit,
    onPickFrom: () -> Unit,
    onPickTo: () -> Unit,
    onSwap: () -> Unit,
) {
    val shape = RoundedCornerShape(24.dp)
    Column(
        modifier = Modifier
            .width(170.dp)
            .height(98.dp)
            .clip(shape)
            .background(
                brush = Brush.verticalGradient(
                    listOf(QalColors.CardLift, QalColors.Card),
                ),
                shape = shape,
            )
            .border(1.dp, QalColors.BorderSoft, shape),
    ) {
        ConversionRow(
            label = "FROM",
            unit = state.from,
            value = state.inputText,
            valueColor = QalColors.Text,
            onValueClick = onEdit,
            onPickUnit = onPickFrom,
            modifier = Modifier.weight(1f),
        )
        SwapDivider(onSwap = onSwap)
        ConversionRow(
            label = "TO",
            unit = state.to,
            value = Units.format(state.result),
            valueColor = QalColors.TealSoft,
            onValueClick = null,
            onPickUnit = onPickTo,
            modifier = Modifier.weight(1f),
        )
    }
}

@Composable
private fun ConversionRow(
    label: String,
    unit: MeasureUnit,
    value: String,
    valueColor: Color,
    onValueClick: (() -> Unit)?,
    onPickUnit: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 12.dp),
        verticalArrangement = Arrangement.Center,
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .height(20.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            BasicText(
                text = label,
                style = QalType.Label.copy(color = QalColors.Faint),
                maxLines = 1,
            )
            Spacer(Modifier.weight(1f))
            UnitButton(unit = unit, onClick = onPickUnit)
        }
        FittedText(
            text = value,
            color = valueColor,
            modifier = Modifier
                .fillMaxWidth()
                .then(
                    if (onValueClick != null) {
                        Modifier.clickable(onClick = onValueClick)
                    } else {
                        Modifier
                    },
                ),
            maxSp = 21,
            minSp = 12,
            textAlign = TextAlign.Start,
        )
    }
}

@Composable
private fun UnitButton(
    unit: MeasureUnit,
    onClick: () -> Unit,
) {
    val haptics = LocalHapticFeedback.current
    Row(
        modifier = Modifier
            .height(20.dp)
            .clip(RoundedCornerShape(50))
            .background(QalColors.Key)
            .border(1.dp, QalColors.BorderSoft, RoundedCornerShape(50))
            .clickable {
                haptics.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                onClick()
            }
            .semantics(mergeDescendants = true) {
                contentDescription = "Change ${unit.name} unit"
                role = Role.Button
            }
            .padding(horizontal = 7.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        BasicText(
            text = unit.symbol,
            style = QalType.Micro.copy(
                color = QalColors.VioletSoft,
                fontWeight = FontWeight.Bold,
                fontSize = 8.sp,
            ),
            maxLines = 1,
        )
        Spacer(Modifier.width(3.dp))
        GlyphIcon(Glyph.ChevronDown, QalColors.Faint, Modifier.size(8.dp))
    }
}

@Composable
private fun SwapDivider(onSwap: () -> Unit) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(12.dp),
    ) {
        Box(
            modifier = Modifier
                .align(Alignment.CenterStart)
                .fillMaxWidth()
                .padding(end = 38.dp)
                .height(1.dp)
                .background(QalColors.Border),
        )
        CircleIconButton(
            glyph = Glyph.Swap,
            label = "Swap units",
            onClick = onSwap,
            modifier = Modifier
                .align(Alignment.CenterEnd)
                .padding(end = 10.dp),
            size = 26.dp,
            background = QalColors.CardLift,
            tint = QalColors.VioletSoft,
        )
    }
}

@Composable
private fun AmountDialog(
    state: ConverterState,
    onDismiss: () -> Unit,
) {
    val haptics = LocalHapticFeedback.current
    QalDialog(onDismissRequest = onDismiss, topPadding = 18.dp) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 50.dp)
                .height(24.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            CircleIconButton(
                glyph = Glyph.Close,
                label = "Cancel",
                onClick = onDismiss,
                size = 24.dp,
            )
            Spacer(Modifier.weight(1f))
            BasicText(
                text = "AMOUNT",
                style = QalType.Label.copy(color = QalColors.Faint),
                maxLines = 1,
            )
            Spacer(Modifier.weight(1f))
            CircleIconButton(
                glyph = Glyph.Check,
                label = "Done",
                onClick = onDismiss,
                size = 24.dp,
                background = QalColors.Violet,
                tint = QalColors.Ink,
            )
        }

        Spacer(Modifier.height(2.dp))

        Column(
            modifier = Modifier.fillMaxWidth(),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.Center,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                BasicText(
                    text = state.from.name,
                    style = QalType.Caption.copy(color = QalColors.Muted),
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Spacer(Modifier.width(5.dp))
                Box(
                    modifier = Modifier
                        .height(16.dp)
                        .clip(RoundedCornerShape(50))
                        .background(QalColors.Key)
                        .border(1.dp, QalColors.BorderSoft, RoundedCornerShape(50))
                        .clickable {
                            haptics.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                            state.toggleSign()
                        }
                        .padding(horizontal = 6.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    BasicText(
                        text = "±",
                        style = QalType.Micro.copy(color = QalColors.VioletSoft, fontWeight = FontWeight.Bold),
                        maxLines = 1,
                    )
                }
            }
            FittedText(
                text = state.inputText,
                color = QalColors.Text,
                modifier = Modifier.fillMaxWidth(),
                maxSp = 27,
                minSp = 15,
                textAlign = TextAlign.Center,
            )
            BasicText(
                text = "= ${Units.format(state.result)} ${state.to.symbol}",
                modifier = Modifier.fillMaxWidth(),
                style = QalType.Caption.copy(
                    color = QalColors.TealSoft,
                    textAlign = TextAlign.Center,
                ),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }

        Spacer(Modifier.height(2.dp))

        ConverterPad(
            state = state,
            modifier = Modifier
                .weight(1f)
                .fillMaxWidth()
                .padding(horizontal = 30.dp),
        )
    }
}

@Composable
private fun ConverterPad(
    state: ConverterState,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier,
        verticalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        PadRow(
            keys = listOf("7", "8", "9", "⌫"),
            state = state,
            modifier = Modifier.weight(1f),
        )
        PadRow(
            keys = listOf("4", "5", "6", "C"),
            state = state,
            modifier = Modifier.weight(1f),
        )
        PadRow(
            keys = listOf("1", "2", "3", "."),
            state = state,
            modifier = Modifier.weight(1f),
        )
        Row(
            modifier = Modifier
                .weight(1f)
                .fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            Spacer(Modifier.weight(1f))
            PadKey(
                label = "0",
                description = "Digit 0",
                kind = PadKeyKind.Digit,
                onClick = { state.appendDigit("0") },
                modifier = Modifier
                    .weight(2f)
                    .fillMaxHeight(),
            )
            Spacer(Modifier.weight(1f))
        }
    }
}

@Composable
private fun PadRow(
    keys: List<String>,
    state: ConverterState,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier = modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        keys.forEach { key ->
            val kind = when (key) {
                "⌫" -> PadKeyKind.Backspace
                "C" -> PadKeyKind.Clear
                "." -> PadKeyKind.Decimal
                else -> PadKeyKind.Digit
            }
            val description = when (key) {
                "⌫" -> "Backspace"
                "C" -> "Clear"
                "." -> "Decimal point"
                else -> "Digit $key"
            }
            PadKey(
                label = key,
                description = description,
                kind = kind,
                onClick = {
                    when (key) {
                        "⌫" -> state.backspace()
                        "C" -> state.clearInput()
                        "." -> state.appendDecimal()
                        else -> state.appendDigit(key)
                    }
                },
                modifier = Modifier
                    .weight(1f)
                    .fillMaxHeight(),
            )
        }
    }
}

private enum class PadKeyKind { Digit, Backspace, Clear, Decimal }

@Composable
private fun PadKey(
    label: String,
    description: String,
    kind: PadKeyKind,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val haptics = LocalHapticFeedback.current
    val interaction = remember { MutableInteractionSource() }
    val pressed by interaction.collectIsPressedAsState()
    val scale by animateFloatAsState(
        targetValue = if (pressed) 0.93f else 1f,
        animationSpec = spring(dampingRatio = 0.55f, stiffness = 700f),
        label = "padKeyScale",
    )
    val base = when (kind) {
        PadKeyKind.Clear -> QalColors.Clear
        PadKeyKind.Backspace -> QalColors.KeyLift
        else -> QalColors.Key
    }
    val lifted = when (kind) {
        PadKeyKind.Clear -> QalColors.ClearLift
        else -> QalColors.KeyLift
    }
    val background by animateColorAsState(
        targetValue = if (pressed) lifted else base,
        label = "padKeyBackground",
    )
    val foreground = when (kind) {
        PadKeyKind.Clear -> QalColors.Pink
        PadKeyKind.Backspace -> QalColors.Text
        else -> QalColors.Text
    }
    val shape = RoundedCornerShape(50)
    Box(
        modifier = modifier
            .graphicsLayer {
                scaleX = scale
                scaleY = scale
            }
            .clip(shape)
            .background(background, shape)
            .border(1.dp, Color.White.copy(alpha = 0.045f), shape)
            .clickable(
                interactionSource = interaction,
                indication = null,
            ) {
                haptics.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                onClick()
            }
            .semantics(mergeDescendants = true) {
                contentDescription = description
                role = Role.Button
            },
        contentAlignment = Alignment.Center,
    ) {
        if (kind == PadKeyKind.Backspace) {
            GlyphIcon(Glyph.Backspace, QalColors.Text, Modifier.size(17.dp))
        } else if (kind == PadKeyKind.Clear) {
            BasicText(
                text = "C",
                style = QalType.Button.copy(fontSize = 14.sp, lineHeight = 17.sp, color = foreground),
                maxLines = 1,
            )
        } else {
            BasicText(
                text = label,
                style = QalType.Button.copy(
                    fontSize = if (label == ".") 20.sp else 17.sp,
                    lineHeight = 20.sp,
                    color = foreground,
                ),
                maxLines = 1,
            )
        }
    }
}

@Composable
private fun UnitPickerDialog(
    category: UnitCategory,
    selectedIndex: Int,
    title: String,
    onSelect: (Int) -> Unit,
    onDismiss: () -> Unit,
) {
    val haptics = LocalHapticFeedback.current
    QalDialog(onDismissRequest = onDismiss) {
        DialogHeader(
            title = title,
            subtitle = category.label,
            onClose = onDismiss,
        )
        Spacer(Modifier.height(4.dp))
        ScalingLazyColumn(
            modifier = Modifier.fillMaxSize(),
            state = rememberScalingLazyListState(initialCenterItemIndex = selectedIndex),
            contentPadding = PaddingValues(horizontal = 16.dp, vertical = 6.dp),
            verticalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            items(category.units) { unit ->
                val selected = category.units.indexOf(unit) == selectedIndex
                val shape = RoundedCornerShape(14.dp)
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(38.dp)
                        .clip(shape)
                        .background(
                            if (selected) QalColors.Violet.copy(alpha = 0.15f) else QalColors.Card,
                        )
                        .border(
                            width = 1.dp,
                            color = if (selected) QalColors.Violet.copy(alpha = 0.45f) else QalColors.BorderSoft,
                            shape = shape,
                        )
                        .clickable {
                            haptics.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                            onSelect(category.units.indexOf(unit))
                        }
                        .padding(horizontal = 10.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    BasicText(
                        text = unit.symbol,
                        modifier = Modifier.width(42.dp),
                        style = QalType.Numeric.copy(color = QalColors.VioletSoft),
                        maxLines = 1,
                    )
                    BasicText(
                        text = unit.name,
                        modifier = Modifier.weight(1f),
                        style = QalType.Caption.copy(color = QalColors.Muted),
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    if (selected) {
                        GlyphIcon(Glyph.Check, QalColors.Teal, Modifier.size(12.dp))
                    }
                }
            }
            item {
                Spacer(Modifier.height(24.dp))
            }
        }
    }
}
