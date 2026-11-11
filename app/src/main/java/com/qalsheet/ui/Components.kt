package com.qalsheet.ui

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.wear.compose.foundation.hierarchicalFocusGroup

enum class Glyph {
    Calculator,
    Convert,
    History,
    Close,
    Check,
    Swap,
    ChevronDown,
    Backspace,
}

private val tabLabels = listOf("Calculator", "Converter")

@Composable
fun GlyphIcon(
    glyph: Glyph,
    tint: Color,
    modifier: Modifier = Modifier,
) {
    Canvas(modifier) {
        val w = size.width
        val h = size.height
        val strokeWidth = w * 0.095f
        val stroke = Stroke(width = strokeWidth, cap = StrokeCap.Round, join = StrokeJoin.Round)
        when (glyph) {
            Glyph.Calculator -> {
                drawRoundRect(
                    color = tint,
                    topLeft = Offset(w * 0.20f, h * 0.09f),
                    size = Size(w * 0.60f, h * 0.82f),
                    cornerRadius = CornerRadius(w * 0.14f),
                    style = stroke,
                )
                drawRoundRect(
                    color = tint,
                    topLeft = Offset(w * 0.31f, h * 0.21f),
                    size = Size(w * 0.38f, h * 0.13f),
                    cornerRadius = CornerRadius(w * 0.035f),
                )
                val dots = listOf(
                    0.33f to 0.47f, 0.50f to 0.47f, 0.67f to 0.47f,
                    0.33f to 0.64f, 0.50f to 0.64f, 0.67f to 0.64f,
                    0.33f to 0.80f, 0.50f to 0.80f,
                )
                dots.forEach { (x, y) ->
                    drawCircle(color = tint, radius = w * 0.028f, center = Offset(w * x, h * y))
                }
            }

            Glyph.Convert -> {
                drawLine(tint, Offset(w * 0.17f, h * 0.38f), Offset(w * 0.83f, h * 0.38f), strokeWidth, StrokeCap.Round)
                drawLine(tint, Offset(w * 0.70f, h * 0.25f), Offset(w * 0.83f, h * 0.38f), strokeWidth, StrokeCap.Round)
                drawLine(tint, Offset(w * 0.70f, h * 0.51f), Offset(w * 0.83f, h * 0.38f), strokeWidth, StrokeCap.Round)
                drawLine(tint, Offset(w * 0.83f, h * 0.66f), Offset(w * 0.17f, h * 0.66f), strokeWidth, StrokeCap.Round)
                drawLine(tint, Offset(w * 0.30f, h * 0.53f), Offset(w * 0.17f, h * 0.66f), strokeWidth, StrokeCap.Round)
                drawLine(tint, Offset(w * 0.30f, h * 0.79f), Offset(w * 0.17f, h * 0.66f), strokeWidth, StrokeCap.Round)
            }

            Glyph.History -> {
                drawCircle(
                    color = tint,
                    radius = w * 0.32f,
                    center = Offset(w * 0.5f, h * 0.52f),
                    style = stroke,
                )
                drawLine(
                    tint,
                    Offset(w * 0.5f, h * 0.52f),
                    Offset(w * 0.5f, h * 0.32f),
                    strokeWidth,
                    StrokeCap.Round,
                )
                drawLine(
                    tint,
                    Offset(w * 0.5f, h * 0.52f),
                    Offset(w * 0.65f, h * 0.59f),
                    strokeWidth,
                    StrokeCap.Round,
                )
            }

            Glyph.Close -> {
                drawLine(tint, Offset(w * 0.27f, h * 0.27f), Offset(w * 0.73f, h * 0.73f), strokeWidth, StrokeCap.Round)
                drawLine(tint, Offset(w * 0.73f, h * 0.27f), Offset(w * 0.27f, h * 0.73f), strokeWidth, StrokeCap.Round)
            }

            Glyph.Check -> {
                drawLine(tint, Offset(w * 0.20f, h * 0.52f), Offset(w * 0.42f, h * 0.73f), strokeWidth, StrokeCap.Round)
                drawLine(tint, Offset(w * 0.42f, h * 0.73f), Offset(w * 0.80f, h * 0.27f), strokeWidth, StrokeCap.Round)
            }

            Glyph.Swap -> {
                drawLine(tint, Offset(w * 0.38f, h * 0.20f), Offset(w * 0.38f, h * 0.80f), strokeWidth, StrokeCap.Round)
                drawLine(tint, Offset(w * 0.38f, h * 0.80f), Offset(w * 0.27f, h * 0.68f), strokeWidth, StrokeCap.Round)
                drawLine(tint, Offset(w * 0.38f, h * 0.80f), Offset(w * 0.49f, h * 0.68f), strokeWidth, StrokeCap.Round)
                drawLine(tint, Offset(w * 0.62f, h * 0.80f), Offset(w * 0.62f, h * 0.20f), strokeWidth, StrokeCap.Round)
                drawLine(tint, Offset(w * 0.62f, h * 0.20f), Offset(w * 0.51f, h * 0.32f), strokeWidth, StrokeCap.Round)
                drawLine(tint, Offset(w * 0.62f, h * 0.20f), Offset(w * 0.73f, h * 0.32f), strokeWidth, StrokeCap.Round)
            }

            Glyph.ChevronDown -> {
                drawLine(tint, Offset(w * 0.27f, h * 0.40f), Offset(w * 0.50f, h * 0.63f), strokeWidth, StrokeCap.Round)
                drawLine(tint, Offset(w * 0.50f, h * 0.63f), Offset(w * 0.73f, h * 0.40f), strokeWidth, StrokeCap.Round)
            }

            Glyph.Backspace -> {
                val shape = Path().apply {
                    moveTo(w * 0.42f, h * 0.18f)
                    lineTo(w * 0.87f, h * 0.18f)
                    lineTo(w * 0.87f, h * 0.82f)
                    lineTo(w * 0.42f, h * 0.82f)
                    lineTo(w * 0.13f, h * 0.50f)
                    close()
                }
                drawPath(shape, color = tint, style = stroke)
                drawLine(tint, Offset(w * 0.52f, h * 0.37f), Offset(w * 0.72f, h * 0.63f), strokeWidth, StrokeCap.Round)
                drawLine(tint, Offset(w * 0.72f, h * 0.37f), Offset(w * 0.52f, h * 0.63f), strokeWidth, StrokeCap.Round)
            }
        }
    }
}

@Composable
fun QalTabs(
    selectedPage: Int,
    onSelect: (Int) -> Unit,
    modifier: Modifier = Modifier,
) {
    val haptics = LocalHapticFeedback.current
    Row(
        modifier = modifier,
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        tabLabels.forEachIndexed { index, label ->
            val isActive = selectedPage == index
            val background by animateColorAsState(
                targetValue = if (isActive) QalColors.Violet else Color.Transparent,
                animationSpec = tween(180),
                label = "tabBackground",
            )
            val scale by animateFloatAsState(
                targetValue = if (isActive) 1f else 0.90f,
                animationSpec = tween(180),
                label = "tabScale",
            )
            val interaction = remember { MutableInteractionSource() }
            Box(
                modifier = Modifier
                    .size(30.dp)
                    .graphicsLayer {
                        scaleX = scale
                        scaleY = scale
                    }
                    .clip(CircleShape)
                    .background(background)
                    .clickable(
                        interactionSource = interaction,
                        indication = null,
                        onClick = {
                            haptics.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                            onSelect(index)
                        },
                    )
                    .semantics(mergeDescendants = true) {
                        contentDescription = label
                        role = Role.Tab
                        selected = isActive
                    },
                contentAlignment = Alignment.Center,
            ) {
                GlyphIcon(
                    glyph = if (index == 0) Glyph.Calculator else Glyph.Convert,
                    tint = if (isActive) Color.White else QalColors.Muted,
                    modifier = Modifier.size(18.dp),
                )
            }
        }
    }
}

@Composable
fun QalDialog(
    onDismissRequest: () -> Unit,
    topPadding: Dp = 26.dp,
    content: @Composable ColumnScope.() -> Unit,
) {
    Dialog(
        onDismissRequest = onDismissRequest,
        properties = DialogProperties(
            dismissOnBackPress = true,
            dismissOnClickOutside = true,
            usePlatformDefaultWidth = false,
            decorFitsSystemWindows = false,
        ),
    ) {
        QalScreen {
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(top = topPadding, bottom = 8.dp)
                    .hierarchicalFocusGroup(active = true),
                content = content,
            )
        }
    }
}

@Composable
fun DialogHeader(
    title: String,
    onClose: () -> Unit,
    subtitle: String? = null,
    action: (@Composable () -> Unit)? = null,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 34.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(modifier = Modifier.weight(1f)) {
            androidx.compose.foundation.text.BasicText(
                text = title,
                style = QalType.Title.copy(color = QalColors.Text),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            if (subtitle != null) {
                androidx.compose.foundation.text.BasicText(
                    text = subtitle,
                    style = QalType.Micro.copy(color = QalColors.Muted),
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
        if (action != null) {
            action()
            Spacer(Modifier.width(6.dp))
        }
        CircleIconButton(
            glyph = Glyph.Close,
            label = "Close",
            onClick = onClose,
        )
    }
}

@Composable
fun CircleIconButton(
    glyph: Glyph,
    label: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    size: Dp = 28.dp,
    background: Color = Color.White.copy(alpha = 0.06f),
    tint: Color = QalColors.Text,
) {
    val haptics = LocalHapticFeedback.current
    Box(
        modifier = modifier
            .size(size)
            .clip(CircleShape)
            .background(background)
            .border(1.dp, QalColors.BorderSoft, CircleShape)
            .clickable {
                haptics.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                onClick()
            }
            .semantics(mergeDescendants = true) {
                contentDescription = label
                role = Role.Button
            },
        contentAlignment = Alignment.Center,
    ) {
        GlyphIcon(glyph, tint, Modifier.size(size * 0.48f))
    }
}

@Composable
fun QalPill(
    text: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    selected: Boolean = false,
    accent: Boolean = false,
) {
    val haptics = LocalHapticFeedback.current
    val background = when {
        selected -> QalColors.Violet
        accent -> QalColors.Violet.copy(alpha = 0.16f)
        else -> QalColors.CardLift
    }
    val foreground = when {
        selected -> QalColors.Ink
        accent -> QalColors.VioletSoft
        else -> QalColors.Muted
    }
    Box(
        modifier = modifier
            .height(24.dp)
            .clip(RoundedCornerShape(50))
            .background(background)
            .border(
                width = 1.dp,
                color = if (selected) Color.Transparent else QalColors.BorderSoft,
                shape = RoundedCornerShape(50),
            )
            .clickable {
                haptics.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                onClick()
            }
            .padding(horizontal = 9.dp),
        contentAlignment = Alignment.Center,
    ) {
        androidx.compose.foundation.text.BasicText(
            text = text,
            style = QalType.Caption.copy(
                color = foreground,
                fontWeight = FontWeight.SemiBold,
            ),
            maxLines = 1,
        )
    }
}

@Composable
fun QalPageSwitch(
    scientific: Boolean,
    onSelect: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
) {
    val haptics = LocalHapticFeedback.current
    Row(
        modifier = modifier
            .height(18.dp)
            .clip(RoundedCornerShape(50))
            .background(QalColors.CardLift)
            .border(1.dp, QalColors.BorderSoft, RoundedCornerShape(50))
            .padding(1.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        PageSwitchSegment(
            label = "123",
            active = !scientific,
            description = "Basic keypad",
            onClick = {
                haptics.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                onSelect(false)
            },
        )
        PageSwitchSegment(
            label = "fx",
            active = scientific,
            description = "Scientific functions",
            onClick = {
                haptics.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                onSelect(true)
            },
        )
    }
}

@Composable
private fun PageSwitchSegment(
    label: String,
    active: Boolean,
    description: String,
    onClick: () -> Unit,
) {
    Box(
        modifier = Modifier
            .width(26.dp)
            .height(16.dp)
            .clip(RoundedCornerShape(50))
            .background(if (active) QalColors.Violet else Color.Transparent)
            .clickable(onClick = onClick)
            .semantics(mergeDescendants = true) {
                contentDescription = description
                role = Role.Tab
                selected = active
            },
        contentAlignment = Alignment.Center,
    ) {
        androidx.compose.foundation.text.BasicText(
            text = label,
            style = QalType.Micro.copy(
                color = if (active) QalColors.Ink else QalColors.Muted,
                fontWeight = FontWeight.Bold,
                fontSize = 7.sp,
                letterSpacing = 0.2.sp,
            ),
            maxLines = 1,
        )
    }
}

@Composable
fun QalModeTag(
    text: String,
    active: Boolean,
    description: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val haptics = LocalHapticFeedback.current
    Box(
        modifier = modifier
            .width(30.dp)
            .height(18.dp)
            .clip(RoundedCornerShape(50))
            .background(
                if (active) QalColors.Teal.copy(alpha = 0.16f) else QalColors.CardLift,
            )
            .border(
                width = 1.dp,
                color = if (active) QalColors.Teal.copy(alpha = 0.28f) else QalColors.BorderSoft,
                shape = RoundedCornerShape(50),
            )
            .clickable {
                haptics.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                onClick()
            }
            .semantics(mergeDescendants = true) {
                contentDescription = description
                role = Role.Button
            },
        contentAlignment = Alignment.Center,
    ) {
        androidx.compose.foundation.text.BasicText(
            text = text,
            style = QalType.Micro.copy(
                color = if (active) QalColors.TealSoft else QalColors.VioletSoft,
                fontWeight = FontWeight.Bold,
                fontSize = 7.sp,
                letterSpacing = 0.2.sp,
            ),
            maxLines = 1,
        )
    }
}

@Composable
fun FittedText(
    text: String,
    color: Color,
    modifier: Modifier = Modifier,
    maxSp: Int,
    minSp: Int,
    fontWeight: FontWeight = FontWeight.ExtraBold,
    textAlign: TextAlign = TextAlign.End,
) {
    val size = when {
        text.length <= 7 -> maxSp
        text.length <= 10 -> maxSp - 4
        text.length <= 13 -> maxSp - 7
        text.length <= 17 -> maxSp - 10
        else -> minSp
    }.coerceAtLeast(minSp)
    androidx.compose.foundation.text.BasicText(
        text = text,
        modifier = modifier,
        style = TextStyle(
            fontFamily = Nunito,
            fontWeight = fontWeight,
            fontSize = size.sp,
            lineHeight = (size + 3).sp,
            letterSpacing = (-0.3).sp,
            color = color,
            textAlign = textAlign,
        ),
        maxLines = 1,
        softWrap = false,
        overflow = TextOverflow.Ellipsis,
    )
}
