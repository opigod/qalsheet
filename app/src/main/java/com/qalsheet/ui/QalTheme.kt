package com.qalsheet.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import com.qalsheet.R

/**
 * QalSheet's palette. It is deliberately a fixed dark theme: the reference app
 * (QuickCalc) uses a black watch face with slate keys, a violet action key and a
 * teal accent, with Material You colours on top only when the watch supports it.
 */
object QalColors {
    val Ink = Color(0xFF07060B)
    val InkLift = Color(0xFF100C18)
    val Card = Color(0xFF17141F)
    val CardLift = Color(0xFF211D2B)
    val Key = Color(0xFF24212E)
    val KeyLift = Color(0xFF302C3D)
    val Operator = Color(0xFF35304A)
    val OperatorLift = Color(0xFF433C5C)
    val Clear = Color(0xFF3F2433)
    val ClearLift = Color(0xFF512D42)

    val Border = Color(0xFF342F41)
    val BorderSoft = Color(0x14FFFFFF)
    val Text = Color(0xFFF7F3FF)
    val Muted = Color(0xFFA39BB7)
    val Faint = Color(0xFF756C88)

    val Violet = Color(0xFF8B5CF6)
    val VioletSoft = Color(0xFFC4B5FD)
    val VioletDeep = Color(0xFF4F378B)
    val Slate = Color(0xFF444559)
    val Teal = Color(0xFF2DD4BF)
    val TealSoft = Color(0xFF99F6E4)
    val Pink = Color(0xFFFF9EC4)
    val Error = Color(0xFFFF6B81)
}

val Nunito = FontFamily(
    Font(R.font.nunito_regular, FontWeight.Normal),
    Font(R.font.nunito_medium, FontWeight.Medium),
    Font(R.font.nunito_semibold, FontWeight.SemiBold),
    Font(R.font.nunito_bold, FontWeight.Bold),
    Font(R.font.nunito_extrabold, FontWeight.ExtraBold),
)

object QalType {
    val Display = TextStyle(
        fontFamily = Nunito,
        fontWeight = FontWeight.ExtraBold,
        fontSize = 30.sp,
        lineHeight = 33.sp,
        letterSpacing = (-0.6).sp,
    )
    val DisplaySmall = TextStyle(
        fontFamily = Nunito,
        fontWeight = FontWeight.ExtraBold,
        fontSize = 23.sp,
        lineHeight = 26.sp,
        letterSpacing = (-0.3).sp,
    )
    val Title = TextStyle(
        fontFamily = Nunito,
        fontWeight = FontWeight.Bold,
        fontSize = 14.sp,
        lineHeight = 17.sp,
    )
    val TitleSmall = TextStyle(
        fontFamily = Nunito,
        fontWeight = FontWeight.Bold,
        fontSize = 12.sp,
        lineHeight = 15.sp,
    )
    val Body = TextStyle(
        fontFamily = Nunito,
        fontWeight = FontWeight.Medium,
        fontSize = 10.sp,
        lineHeight = 13.sp,
    )
    val Caption = TextStyle(
        fontFamily = Nunito,
        fontWeight = FontWeight.Medium,
        fontSize = 8.sp,
        lineHeight = 11.sp,
    )
    val Micro = TextStyle(
        fontFamily = Nunito,
        fontWeight = FontWeight.SemiBold,
        fontSize = 7.sp,
        lineHeight = 9.sp,
        letterSpacing = 0.2.sp,
    )
    val Label = TextStyle(
        fontFamily = Nunito,
        fontWeight = FontWeight.Bold,
        fontSize = 7.sp,
        lineHeight = 9.sp,
        letterSpacing = 1.2.sp,
    )
    val Button = TextStyle(
        fontFamily = Nunito,
        fontWeight = FontWeight.ExtraBold,
        fontSize = 17.sp,
        lineHeight = 20.sp,
    )
    val Operator = TextStyle(
        fontFamily = Nunito,
        fontWeight = FontWeight.Bold,
        fontSize = 19.sp,
        lineHeight = 22.sp,
    )
    val Numeric = TextStyle(
        fontFamily = Nunito,
        fontWeight = FontWeight.Bold,
        fontSize = 12.sp,
        lineHeight = 15.sp,
    )
}

@Composable
fun QalScreen(
    modifier: Modifier = Modifier,
    content: @Composable BoxScope.() -> Unit,
) {
    Box(
        modifier = modifier
            .fillMaxSize()
            .drawBehind {
                drawRect(
                    brush = Brush.verticalGradient(
                        0f to QalColors.InkLift,
                        0.48f to QalColors.Ink,
                        1f to Color(0xFF06040A),
                    ),
                )
                val topGlow = Offset(size.width * 0.5f, -size.height * 0.10f)
                val topRadius = size.width * 0.82f
                drawCircle(
                    brush = Brush.radialGradient(
                        colors = listOf(QalColors.Violet.copy(alpha = 0.17f), Color.Transparent),
                        center = topGlow,
                        radius = topRadius,
                    ),
                    radius = topRadius,
                    center = topGlow,
                )
                val bottomGlow = Offset(-size.width * 0.08f, size.height * 0.94f)
                val bottomRadius = size.width * 0.66f
                drawCircle(
                    brush = Brush.radialGradient(
                        colors = listOf(QalColors.Teal.copy(alpha = 0.055f), Color.Transparent),
                        center = bottomGlow,
                        radius = bottomRadius,
                    ),
                    radius = bottomRadius,
                    center = bottomGlow,
                )
            },
        content = content,
    )
}
