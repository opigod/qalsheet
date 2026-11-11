package com.qalsheet.calc

import androidx.compose.animation.Crossfade
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
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
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
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
import com.qalsheet.ui.DialogHeader
import com.qalsheet.ui.FittedText
import com.qalsheet.ui.Glyph
import com.qalsheet.ui.GlyphIcon
import com.qalsheet.ui.QalColors
import com.qalsheet.ui.QalDialog
import com.qalsheet.ui.QalModeTag
import com.qalsheet.ui.QalPageSwitch
import com.qalsheet.ui.QalPill
import com.qalsheet.ui.QalType

private enum class KeyKind {
    Digit,
    Operator,
    Clear,
    Function,
    Equals,
    Backspace,
}

private data class KeySpec(
    val label: String,
    val kind: KeyKind,
    val description: String,
)

private val keyRows: List<List<KeySpec?>> = listOf(
    listOf(
        KeySpec("7", KeyKind.Digit, "Digit 7"),
        KeySpec("8", KeyKind.Digit, "Digit 8"),
        KeySpec("9", KeyKind.Digit, "Digit 9"),
        KeySpec("÷", KeyKind.Operator, "Divide"),
        KeySpec("C", KeyKind.Clear, "Clear"),
    ),
    listOf(
        KeySpec("4", KeyKind.Digit, "Digit 4"),
        KeySpec("5", KeyKind.Digit, "Digit 5"),
        KeySpec("6", KeyKind.Digit, "Digit 6"),
        KeySpec("×", KeyKind.Operator, "Multiply"),
        KeySpec("⌫", KeyKind.Backspace, "Backspace"),
    ),
    listOf(
        KeySpec("1", KeyKind.Digit, "Digit 1"),
        KeySpec("2", KeyKind.Digit, "Digit 2"),
        KeySpec("3", KeyKind.Digit, "Digit 3"),
        KeySpec("−", KeyKind.Operator, "Minus"),
        KeySpec("=", KeyKind.Equals, "Equals"),
    ),
    listOf(
        null,
        KeySpec("0", KeyKind.Digit, "Digit 0"),
        KeySpec(".", KeyKind.Digit, "Decimal point"),
        KeySpec("+", KeyKind.Operator, "Plus"),
        null,
    ),
)

@Composable
fun CalculatorPage(
    state: CalculatorState,
    modifier: Modifier = Modifier,
) {
    var showHistory by remember { mutableStateOf(false) }

    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(horizontal = 6.dp),
    ) {
        CalculatorDisplay(
            state = state,
            onHistory = { showHistory = true },
            onPageChange = state::showScientificPage,
            onToggleAngle = state::toggleAngleMode,
            modifier = Modifier
                .weight(0.30f)
                .fillMaxWidth()
                .padding(horizontal = 14.dp),
        )
        Spacer(Modifier.height(2.dp))
        Crossfade(
            targetState = state.showScientific,
            animationSpec = tween(160),
            label = "keypadPage",
            modifier = Modifier
                .weight(0.70f)
                .fillMaxWidth()
                .padding(horizontal = 14.dp),
        ) { scientific ->
            if (scientific) {
                ScientificKeypad(
                    state = state,
                    modifier = Modifier.fillMaxSize(),
                )
            } else {
                CalculatorKeypad(
                    state = state,
                    modifier = Modifier.fillMaxSize(),
                )
            }
        }
    }

    if (showHistory) {
        HistoryDialog(
            history = state.history,
            onDismiss = { showHistory = false },
            onLoad = { entry ->
                state.loadHistory(entry)
                showHistory = false
            },
            onClear = { state.clearHistory() },
        )
    }
}

@Composable
private fun CalculatorDisplay(
    state: CalculatorState,
    onHistory: () -> Unit,
    onPageChange: (Boolean) -> Unit,
    onToggleAngle: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val haptics = LocalHapticFeedback.current
    val topText = state.topText
    val topColor = when {
        state.topIsPreview -> QalColors.VioletSoft
        topText == "qalsheet" -> QalColors.VioletSoft
        else -> QalColors.Muted
    }
    val topStyle = if (topText == "qalsheet") {
        QalType.Label.copy(color = topColor, letterSpacing = 1.6.sp)
    } else {
        QalType.Caption.copy(color = topColor)
    }

    Column(
        modifier = modifier,
        verticalArrangement = Arrangement.SpaceBetween,
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 8.dp)
                .height(20.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            QalPageSwitch(
                scientific = state.showScientific,
                onSelect = onPageChange,
            )
            Spacer(Modifier.width(4.dp))
            if (topText.isNotEmpty()) {
                BasicText(
                    text = topText,
                    modifier = Modifier.weight(1f),
                    style = topStyle.copy(textAlign = TextAlign.End),
                    maxLines = 1,
                    softWrap = false,
                    overflow = TextOverflow.Ellipsis,
                )
            } else {
                Spacer(Modifier.weight(1f))
            }
            Spacer(Modifier.width(4.dp))
            QalModeTag(
                text = state.angleMode.label,
                active = state.angleMode == AngleMode.Radians,
                description = "Angle mode ${state.angleMode.label}. Tap to switch.",
                onClick = onToggleAngle,
            )
            if (state.history.isNotEmpty()) {
                Spacer(Modifier.width(4.dp))
                Box(
                    modifier = Modifier
                        .size(20.dp)
                        .clip(CircleShape)
                        .clickable {
                            haptics.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                            onHistory()
                        }
                        .semantics(mergeDescendants = true) {
                            contentDescription = "History"
                            role = Role.Button
                        },
                    contentAlignment = Alignment.Center,
                ) {
                    GlyphIcon(Glyph.History, QalColors.Muted, Modifier.size(13.dp))
                }
            }
        }

        FittedText(
            text = state.displayText,
            color = if (state.isError) QalColors.Error else QalColors.Text,
            modifier = Modifier.fillMaxWidth(),
            maxSp = 30,
            minSp = 15,
        )
    }
}

@Composable
private fun CalculatorKeypad(
    state: CalculatorState,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier,
        verticalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        keyRows.forEach { row ->
            Row(
                modifier = Modifier
                    .weight(1f)
                    .fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                row.forEach { spec ->
                    if (spec == null) {
                        Spacer(Modifier.weight(1f))
                    } else {
                        CalculatorKey(
                            spec = spec,
                            onClick = { state.press(spec.label) },
                            modifier = Modifier
                                .weight(1f)
                                .fillMaxHeight(),
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun ScientificKeypad(
    state: CalculatorState,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier,
        verticalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        scienceRows.forEach { row ->
            Row(
                modifier = Modifier
                    .weight(1f)
                    .fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                row.forEach { spec ->
                    ScienceKey(
                        spec = spec,
                        onClick = { state.applyScientific(spec.function) },
                        modifier = Modifier
                            .weight(1f)
                            .fillMaxHeight(),
                    )
                }
            }
        }
        Row(
            modifier = Modifier
                .weight(1f)
                .fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            Spacer(Modifier.weight(0.5f))
            scienceBottom.forEach { spec ->
                ScienceKey(
                    spec = spec,
                    onClick = { state.applyScientific(spec.function) },
                    modifier = Modifier
                        .weight(1f)
                        .fillMaxHeight(),
                )
            }
            Spacer(Modifier.weight(0.5f))
        }
    }
}

private enum class ScienceKind {
    Math,
    Constant,
    Utility,
}

private data class ScienceSpec(
    val function: ScientificFunction,
    val labelSize: Int,
    val description: String,
    val kind: ScienceKind,
)

private val scienceRows: List<List<ScienceSpec>> = listOf(
    listOf(
        ScienceSpec(ScientificFunction.Sin, 10, "Sine", ScienceKind.Math),
        ScienceSpec(ScientificFunction.Cos, 10, "Cosine", ScienceKind.Math),
        ScienceSpec(ScientificFunction.Tan, 10, "Tangent", ScienceKind.Math),
        ScienceSpec(ScientificFunction.Asin, 8, "Arcsine", ScienceKind.Utility),
        ScienceSpec(ScientificFunction.Acos, 8, "Arccosine", ScienceKind.Utility),
    ),
    listOf(
        ScienceSpec(ScientificFunction.Atan, 8, "Arctangent", ScienceKind.Utility),
        ScienceSpec(ScientificFunction.Ln, 9, "Natural logarithm", ScienceKind.Math),
        ScienceSpec(ScientificFunction.Log, 9, "Log base 10", ScienceKind.Math),
        ScienceSpec(ScientificFunction.Sqrt, 17, "Square root", ScienceKind.Math),
        ScienceSpec(ScientificFunction.Square, 11, "Square", ScienceKind.Math),
    ),
    listOf(
        ScienceSpec(ScientificFunction.Power, 9, "Power", ScienceKind.Math),
        ScienceSpec(ScientificFunction.Pi, 16, "Pi", ScienceKind.Constant),
        ScienceSpec(ScientificFunction.E, 13, "Euler's number", ScienceKind.Constant),
        ScienceSpec(ScientificFunction.OpenParen, 16, "Open parenthesis", ScienceKind.Constant),
        ScienceSpec(ScientificFunction.CloseParen, 16, "Close parenthesis", ScienceKind.Constant),
    ),
)

private val scienceBottom: List<ScienceSpec> = listOf(
    ScienceSpec(ScientificFunction.Factorial, 15, "Factorial", ScienceKind.Utility),
    ScienceSpec(ScientificFunction.Percent, 12, "Percent", ScienceKind.Utility),
    ScienceSpec(ScientificFunction.Sign, 13, "Toggle sign", ScienceKind.Utility),
    ScienceSpec(ScientificFunction.Reciprocal, 9, "Reciprocal", ScienceKind.Utility),
)

@Composable
private fun ScienceKey(
    spec: ScienceSpec,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val haptics = LocalHapticFeedback.current
    val interaction = remember { MutableInteractionSource() }
    val pressed by interaction.collectIsPressedAsState()
    val scale by animateFloatAsState(
        targetValue = if (pressed) 0.93f else 1f,
        animationSpec = spring(
            dampingRatio = 0.55f,
            stiffness = 700f,
        ),
        label = "scienceKeyScale",
    )
    val background by animateColorAsState(
        targetValue = when (spec.kind) {
            ScienceKind.Math -> if (pressed) QalColors.OperatorLift else QalColors.Operator
            ScienceKind.Constant -> if (pressed) QalColors.OperatorLift else QalColors.KeyLift
            ScienceKind.Utility -> if (pressed) QalColors.OperatorLift else QalColors.CardLift
        },
        label = "scienceKeyBackground",
    )
    val foreground = when (spec.kind) {
        ScienceKind.Math, ScienceKind.Utility -> QalColors.VioletSoft
        ScienceKind.Constant -> QalColors.TealSoft
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
                contentDescription = spec.description
                role = Role.Button
            },
        contentAlignment = Alignment.Center,
    ) {
        BasicText(
            text = spec.function.label,
            style = QalType.Button.copy(
                fontSize = spec.labelSize.sp,
                lineHeight = (spec.labelSize + 2).sp,
                color = foreground,
            ),
            maxLines = 1,
        )
    }
}

@Composable
private fun CalculatorKey(
    spec: KeySpec,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val haptics = LocalHapticFeedback.current
    val interaction = remember { MutableInteractionSource() }
    val pressed by interaction.collectIsPressedAsState()
    val scale by animateFloatAsState(
        targetValue = if (pressed) 0.93f else 1f,
        animationSpec = spring(
            dampingRatio = 0.55f,
            stiffness = 700f,
        ),
        label = "keyScale",
    )

    val background = when (spec.kind) {
        KeyKind.Digit -> QalColors.Key
        KeyKind.Operator -> QalColors.Operator
        KeyKind.Clear -> QalColors.Clear
        KeyKind.Function, KeyKind.Backspace -> QalColors.KeyLift
        KeyKind.Equals -> QalColors.Violet
    }
    val pressedBackground = when (spec.kind) {
        KeyKind.Digit -> QalColors.KeyLift
        KeyKind.Operator -> QalColors.OperatorLift
        KeyKind.Clear -> QalColors.ClearLift
        KeyKind.Function, KeyKind.Backspace -> QalColors.OperatorLift
        KeyKind.Equals -> QalColors.VioletSoft
    }
    val animatedBackground by animateColorAsState(
        targetValue = if (pressed) pressedBackground else background,
        label = "keyBackground",
    )
    val contentColor = when (spec.kind) {
        KeyKind.Digit, KeyKind.Function, KeyKind.Backspace -> QalColors.Text
        KeyKind.Operator -> QalColors.VioletSoft
        KeyKind.Clear -> QalColors.Pink
        KeyKind.Equals -> QalColors.Ink
    }
    val shape = RoundedCornerShape(50)

    Box(
        modifier = modifier
            .graphicsLayer {
                scaleX = scale
                scaleY = scale
            }
            .clip(shape)
            .then(
                if (spec.kind == KeyKind.Equals) {
                    Modifier.background(
                        brush = Brush.linearGradient(
                            listOf(QalColors.Violet, QalColors.VioletSoft),
                        ),
                        shape = shape,
                    )
                } else {
                    Modifier.background(animatedBackground, shape)
                },
            )
            .border(
                width = 1.dp,
                color = if (spec.kind == KeyKind.Equals) {
                    Color.White.copy(alpha = 0.18f)
                } else {
                    Color.White.copy(alpha = 0.045f)
                },
                shape = shape,
            )
            .clickable(
                interactionSource = interaction,
                indication = null,
            ) {
                haptics.performHapticFeedback(
                    if (spec.kind == KeyKind.Equals) {
                        HapticFeedbackType.LongPress
                    } else {
                        HapticFeedbackType.TextHandleMove
                    },
                )
                onClick()
            }
            .semantics(mergeDescendants = true) {
                contentDescription = spec.description
                role = Role.Button
            },
        contentAlignment = Alignment.Center,
    ) {
        when (spec.kind) {
            KeyKind.Backspace -> GlyphIcon(
                glyph = Glyph.Backspace,
                tint = contentColor,
                modifier = Modifier.size(18.dp),
            )
            KeyKind.Equals -> BasicText(
                text = "=",
                style = QalType.Operator.copy(
                    fontSize = 22.sp,
                    lineHeight = 24.sp,
                    color = contentColor,
                    fontWeight = FontWeight.ExtraBold,
                ),
                maxLines = 1,
            )
            KeyKind.Clear -> BasicText(
                text = "C",
                style = QalType.Button.copy(
                    fontSize = 15.sp,
                    lineHeight = 18.sp,
                    color = contentColor,
                ),
                maxLines = 1,
            )
            KeyKind.Operator -> BasicText(
                text = spec.label,
                style = QalType.Operator.copy(color = contentColor),
                maxLines = 1,
            )
            else -> BasicText(
                text = spec.label,
                style = QalType.Button.copy(color = contentColor),
                maxLines = 1,
            )
        }
    }
}

@Composable
private fun HistoryDialog(
    history: List<HistoryEntry>,
    onDismiss: () -> Unit,
    onLoad: (HistoryEntry) -> Unit,
    onClear: () -> Unit,
) {
    val haptics = LocalHapticFeedback.current
    QalDialog(onDismissRequest = onDismiss) {
        DialogHeader(
            title = "History",
            subtitle = if (history.isEmpty()) {
                "Nothing yet"
            } else if (history.size == 1) {
                "1 calculation"
            } else {
                "${history.size} calculations"
            },
            onClose = onDismiss,
            action = if (history.isNotEmpty()) {
                {
                    QalPill(
                        text = "Clear",
                        onClick = {
                            haptics.performHapticFeedback(HapticFeedbackType.LongPress)
                            onClear()
                        },
                        accent = true,
                    )
                }
            } else {
                null
            },
        )
        Spacer(Modifier.height(4.dp))
        if (history.isEmpty()) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(horizontal = 34.dp),
                contentAlignment = Alignment.Center,
            ) {
                BasicText(
                    text = "No calculations yet.\nYour results will appear here.",
                    style = QalType.Caption.copy(
                        color = QalColors.Muted,
                        textAlign = TextAlign.Center,
                    ),
                )
            }
        } else {
            ScalingLazyColumn(
                modifier = Modifier.fillMaxSize(),
                state = rememberScalingLazyListState(),
                contentPadding = PaddingValues(horizontal = 16.dp, vertical = 6.dp),
                verticalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                items(history) { entry ->
                    HistoryRow(
                        entry = entry,
                        onClick = {
                            haptics.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                            onLoad(entry)
                        },
                    )
                }
                item {
                    Spacer(Modifier.height(24.dp))
                }
            }
        }
    }
}

@Composable
private fun HistoryRow(
    entry: HistoryEntry,
    onClick: () -> Unit,
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .height(38.dp)
            .clip(RoundedCornerShape(14.dp))
            .background(QalColors.Card)
            .border(1.dp, QalColors.BorderSoft, RoundedCornerShape(14.dp))
            .clickable(onClick = onClick)
            .padding(horizontal = 10.dp, vertical = 5.dp),
        verticalArrangement = Arrangement.Center,
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            BasicText(
                text = entry.expression,
                modifier = Modifier.weight(1f),
                style = QalType.Micro.copy(color = QalColors.Muted),
                maxLines = 1,
                softWrap = false,
                overflow = TextOverflow.Ellipsis,
            )
            if (entry.angleMode != null) {
                Spacer(Modifier.width(4.dp))
                BasicText(
                    text = entry.angleMode.label,
                    style = QalType.Micro.copy(
                        color = QalColors.VioletSoft,
                        fontSize = 7.sp,
                    ),
                    maxLines = 1,
                )
            }
        }
        BasicText(
            text = entry.result,
            style = QalType.TitleSmall.copy(color = QalColors.Text),
            maxLines = 1,
            softWrap = false,
            overflow = TextOverflow.Ellipsis,
        )
    }
}
