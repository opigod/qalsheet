package com.qalsheet.algebra

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
fun AlgebraPage(
    state: AlgebraState,
    modifier: Modifier = Modifier,
) {
    var showLetters by remember { mutableStateOf(false) }

    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(horizontal = 6.dp),
    ) {
        AlgebraDisplay(
            state = state,
            onChooseVariable = { showLetters = true },
            modifier = Modifier
                .weight(0.34f)
                .fillMaxWidth()
                .padding(horizontal = 14.dp),
        )
        Spacer(Modifier.height(2.dp))
        AlgebraKeypad(
            state = state,
            modifier = Modifier
                .weight(0.66f)
                .fillMaxWidth()
                .padding(horizontal = 14.dp),
        )
    }

    if (showLetters) {
        VariablePickerDialog(
            selected = state.variable,
            onSelect = {
                state.selectVariable(it)
                showLetters = false
            },
            onDismiss = { showLetters = false },
        )
    }
}

@Composable
private fun AlgebraDisplay(
    state: AlgebraState,
    onChooseVariable: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val haptics = LocalHapticFeedback.current
    val displayColor = when {
        state.isError -> QalColors.Error
        state.isSolution -> QalColors.TealSoft
        state.isWarning -> QalColors.Pink
        else -> QalColors.Text
    }
    val hintColor = when {
        state.isError -> QalColors.Error.copy(alpha = 0.85f)
        state.isWarning -> QalColors.Pink.copy(alpha = 0.85f)
        else -> QalColors.Muted
    }

    Column(modifier = modifier) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .height(24.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            ClearKey(
                onClick = {
                    haptics.performHapticFeedback(HapticFeedbackType.LongPress)
                    state.press("C")
                },
            )
            Spacer(Modifier.width(3.dp))
            CircleIconButton(
                glyph = Glyph.Backspace,
                label = "Backspace",
                onClick = { state.press("⌫") },
                size = 24.dp,
            )
            Spacer(Modifier.width(4.dp))
            VariableButton(
                letter = state.variable,
                onClick = onChooseVariable,
            )
            Spacer(Modifier.weight(1f))
            QalPill(
                text = "Solve",
                onClick = state::solve,
                selected = true,
            )
        }
        Spacer(Modifier.height(1.dp))
        BasicText(
            text = state.hintText,
            modifier = Modifier.fillMaxWidth(),
            style = QalType.Micro.copy(
                color = hintColor,
                textAlign = TextAlign.Center,
            ),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        FittedText(
            text = state.displayText,
            color = displayColor,
            modifier = Modifier.fillMaxWidth(),
            maxSp = 22,
            minSp = 11,
            textAlign = TextAlign.Center,
        )
    }
}

@Composable
private fun ClearKey(onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .size(24.dp)
            .clip(CircleShape)
            .background(QalColors.Clear)
            .border(1.dp, QalColors.BorderSoft, CircleShape)
            .clickable(onClick = onClick)
            .semantics(mergeDescendants = true) {
                contentDescription = "Clear equation"
                role = Role.Button
            },
        contentAlignment = Alignment.Center,
    ) {
        BasicText(
            text = "C",
            style = QalType.Button.copy(
                fontSize = 11.sp,
                lineHeight = 13.sp,
                color = QalColors.Pink,
            ),
            maxLines = 1,
        )
    }
}

@Composable
private fun VariableButton(
    letter: Char,
    onClick: () -> Unit,
) {
    val haptics = LocalHapticFeedback.current
    Row(
        modifier = Modifier
            .height(20.dp)
            .clip(RoundedCornerShape(50))
            .background(QalColors.CardLift)
            .border(1.dp, QalColors.BorderSoft, RoundedCornerShape(50))
            .clickable {
                haptics.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                onClick()
            }
            .semantics(mergeDescendants = true) {
                contentDescription = "Solve for $letter. Tap to choose a letter."
                role = Role.Button
            }
            .padding(horizontal = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        BasicText(
            text = letter.toString(),
            style = QalType.Micro.copy(
                color = QalColors.VioletSoft,
                fontWeight = FontWeight.Bold,
                fontSize = 9.sp,
            ),
            maxLines = 1,
        )
        Spacer(Modifier.width(2.dp))
        GlyphIcon(Glyph.ChevronDown, QalColors.Faint, Modifier.size(8.dp))
    }
}

@Composable
private fun AlgebraKeypad(
    state: AlgebraState,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier,
        verticalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        algebraTopRows.forEach { row ->
            Row(
                modifier = Modifier
                    .weight(1f)
                    .fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                row.forEach { spec ->
                    AlgebraKey(
                        spec = spec,
                        state = state,
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
            algebraBottom.forEach { spec ->
                AlgebraKey(
                    spec = spec,
                    state = state,
                    modifier = Modifier
                        .weight(1f)
                        .fillMaxHeight(),
                )
            }
            Spacer(Modifier.weight(0.5f))
        }
    }
}

private enum class AlgebraKeyKind {
    Digit,
    Operator,
    Variable,
    Paren,
    Equals,
}

private data class AlgebraKeySpec(
    val label: String,
    val kind: AlgebraKeyKind,
    val description: String,
)

private val variableKey = AlgebraKeySpec("x", AlgebraKeyKind.Variable, "Variable")

private val algebraTopRows: List<List<AlgebraKeySpec>> = listOf(
    listOf(
        AlgebraKeySpec("7", AlgebraKeyKind.Digit, "Digit 7"),
        AlgebraKeySpec("8", AlgebraKeyKind.Digit, "Digit 8"),
        AlgebraKeySpec("9", AlgebraKeyKind.Digit, "Digit 9"),
        AlgebraKeySpec("÷", AlgebraKeyKind.Operator, "Divide"),
        AlgebraKeySpec("(", AlgebraKeyKind.Paren, "Open parenthesis"),
    ),
    listOf(
        AlgebraKeySpec("4", AlgebraKeyKind.Digit, "Digit 4"),
        AlgebraKeySpec("5", AlgebraKeyKind.Digit, "Digit 5"),
        AlgebraKeySpec("6", AlgebraKeyKind.Digit, "Digit 6"),
        AlgebraKeySpec("×", AlgebraKeyKind.Operator, "Multiply"),
        AlgebraKeySpec(")", AlgebraKeyKind.Paren, "Close parenthesis"),
    ),
    listOf(
        AlgebraKeySpec("1", AlgebraKeyKind.Digit, "Digit 1"),
        AlgebraKeySpec("2", AlgebraKeyKind.Digit, "Digit 2"),
        AlgebraKeySpec("3", AlgebraKeyKind.Digit, "Digit 3"),
        AlgebraKeySpec("−", AlgebraKeyKind.Operator, "Minus"),
        AlgebraKeySpec("=", AlgebraKeyKind.Equals, "Equals"),
    ),
)

private val algebraBottom: List<AlgebraKeySpec> = listOf(
    AlgebraKeySpec("0", AlgebraKeyKind.Digit, "Digit 0"),
    AlgebraKeySpec(".", AlgebraKeyKind.Digit, "Decimal point"),
    variableKey,
    AlgebraKeySpec("+", AlgebraKeyKind.Operator, "Plus"),
)

@Composable
private fun AlgebraKey(
    spec: AlgebraKeySpec,
    state: AlgebraState,
    modifier: Modifier = Modifier,
) {
    val haptics = LocalHapticFeedback.current
    val interaction = remember { MutableInteractionSource() }
    val pressed by interaction.collectIsPressedAsState()
    val scale by animateFloatAsState(
        targetValue = if (pressed) 0.93f else 1f,
        animationSpec = spring(dampingRatio = 0.55f, stiffness = 700f),
        label = "algebraKeyScale",
    )
    val isVariable = spec.kind == AlgebraKeyKind.Variable
    val label = if (isVariable) state.variable.toString() else spec.label
    val description = if (isVariable) "Variable ${state.variable}" else spec.description

    val background by animateColorAsState(
        targetValue = when (spec.kind) {
            AlgebraKeyKind.Digit -> if (pressed) QalColors.KeyLift else QalColors.Key
            AlgebraKeyKind.Operator -> if (pressed) QalColors.OperatorLift else QalColors.Operator
            AlgebraKeyKind.Variable -> if (pressed) QalColors.OperatorLift else QalColors.Operator
            AlgebraKeyKind.Paren -> if (pressed) QalColors.OperatorLift else QalColors.KeyLift
            AlgebraKeyKind.Equals -> QalColors.Violet
        },
        label = "algebraKeyBackground",
    )
    val foreground = when (spec.kind) {
        AlgebraKeyKind.Digit -> QalColors.Text
        AlgebraKeyKind.Operator -> QalColors.VioletSoft
        AlgebraKeyKind.Variable -> QalColors.TealSoft
        AlgebraKeyKind.Paren -> QalColors.Muted
        AlgebraKeyKind.Equals -> QalColors.Ink
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
                if (spec.kind == AlgebraKeyKind.Equals) {
                    Modifier.background(
                        brush = Brush.linearGradient(
                            listOf(QalColors.Violet, QalColors.VioletSoft),
                        ),
                        shape = shape,
                    )
                } else {
                    Modifier.background(background, shape)
                },
            )
            .border(
                width = 1.dp,
                color = if (spec.kind == AlgebraKeyKind.Equals) {
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
                haptics.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                if (isVariable) state.insertVariable() else state.press(spec.label)
            }
            .semantics(mergeDescendants = true) {
                contentDescription = description
                role = Role.Button
            },
        contentAlignment = Alignment.Center,
    ) {
        BasicText(
            text = label,
            style = QalType.Button.copy(
                fontSize = when (spec.kind) {
                    AlgebraKeyKind.Variable -> 18.sp
                    AlgebraKeyKind.Equals -> 20.sp
                    AlgebraKeyKind.Operator -> 19.sp
                    AlgebraKeyKind.Paren -> 17.sp
                    AlgebraKeyKind.Digit -> 17.sp
                },
                lineHeight = 20.sp,
                color = foreground,
            ),
            maxLines = 1,
        )
    }
}

@Composable
private fun VariablePickerDialog(
    selected: Char,
    onSelect: (Char) -> Unit,
    onDismiss: () -> Unit,
) {
    val haptics = LocalHapticFeedback.current
    val letters = remember { ('a'..'z').toList() }
    val rows = remember { letters.chunked(4) }
    val selectedRow = remember(selected) {
        letters.indexOf(selected).coerceAtLeast(0) / 4
    }

    QalDialog(onDismissRequest = onDismiss) {
        DialogHeader(
            title = "Solve for",
            subtitle = "Choose a letter",
            onClose = onDismiss,
        )
        Spacer(Modifier.height(4.dp))
        ScalingLazyColumn(
            modifier = Modifier.fillMaxSize(),
            state = rememberScalingLazyListState(initialCenterItemIndex = selectedRow),
            contentPadding = PaddingValues(horizontal = 16.dp, vertical = 6.dp),
            verticalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            items(rows) { row ->
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(4.dp),
                ) {
                    row.forEach { letter ->
                        LetterTile(
                            letter = letter,
                            selected = letter == selected,
                            onClick = {
                                haptics.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                                onSelect(letter)
                            },
                            modifier = Modifier.weight(1f),
                        )
                    }
                    repeat(4 - row.size) {
                        Spacer(Modifier.weight(1f))
                    }
                }
            }
            item {
                Spacer(Modifier.height(24.dp))
            }
        }
    }
}

@Composable
private fun LetterTile(
    letter: Char,
    selected: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val shape = RoundedCornerShape(12.dp)
    Box(
        modifier = modifier
            .height(34.dp)
            .clip(shape)
            .background(if (selected) QalColors.Violet.copy(alpha = 0.16f) else QalColors.Card)
            .border(
                width = 1.dp,
                color = if (selected) QalColors.Violet.copy(alpha = 0.55f) else QalColors.BorderSoft,
                shape = shape,
            )
            .clickable(onClick = onClick)
            .semantics(mergeDescendants = true) {
                contentDescription = "Letter $letter"
                role = Role.Button
            },
        contentAlignment = Alignment.Center,
    ) {
        BasicText(
            text = letter.toString(),
            style = QalType.TitleSmall.copy(
                color = if (selected) QalColors.VioletSoft else QalColors.Text,
                fontWeight = FontWeight.ExtraBold,
            ),
            maxLines = 1,
        )
    }
}
