package com.qalsheet.chem

import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.PaddingValues
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
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
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
import androidx.wear.compose.foundation.lazy.ScalingLazyColumnDefaults
import androidx.wear.compose.foundation.lazy.items
import androidx.wear.compose.foundation.lazy.rememberScalingLazyListState
import com.qalsheet.ui.DialogHeader
import com.qalsheet.ui.FittedText
import com.qalsheet.ui.FormulaText
import com.qalsheet.ui.Glyph
import com.qalsheet.ui.GlyphIcon
import com.qalsheet.ui.QalColors
import com.qalsheet.ui.QalDialog
import com.qalsheet.ui.QalPill
import com.qalsheet.ui.QalType
import com.qalsheet.ui.SuffixBadge

class ChemistryState {
    var symbol by mutableStateOf("Fe")
        private set
    var oxidation by mutableIntStateOf(3)
        private set

    val element: ChemElement get() = ChemData.find(symbol)
    val state: ChemState get() = element.state(oxidation)

    fun selectElement(symbol: String) {
        val selected = ChemData.find(symbol)
        this.symbol = symbol
        oxidation = selected.states.last().oxidation
    }

    fun selectOxidation(oxidation: Int) {
        if (element.states.any { it.oxidation == oxidation }) {
            this.oxidation = oxidation
        }
    }
}

@Composable
fun ChemistryPage(
    state: ChemistryState,
    modifier: Modifier = Modifier,
) {
    var showPicker by remember { mutableStateOf(false) }
    var showGuide by remember { mutableStateOf(false) }

    val element = state.element
    val oxidationState = state.state
    val examples = remember(element.symbol, oxidationState.oxidation) {
        ChemData.examplesFor(element, oxidationState)
    }
    val suffix = remember(oxidationState.adjective) {
        ChemData.suffixOf(oxidationState.adjective)
    }

    ScalingLazyColumn(
        modifier = modifier.fillMaxSize(),
        state = rememberScalingLazyListState(),
        contentPadding = PaddingValues(horizontal = 10.dp, vertical = 2.dp),
        verticalArrangement = Arrangement.spacedBy(4.dp),
        autoCentering = null,
        scalingParams = ScalingLazyColumnDefaults.scalingParams(
            edgeScale = 1f,
            edgeAlpha = 1f,
            minElementHeight = 1f,
            maxElementHeight = 1f,
        ),
    ) {
        item {
            ElementCard(
                element = element,
                onClick = { showPicker = true },
                onGuide = { showGuide = true },
            )
        }

        item {
            OxidationChips(
                element = element,
                selected = oxidationState.oxidation,
                onSelect = { state.selectOxidation(it) },
            )
        }

        item {
            ResultCard(
                element = element,
                state = oxidationState,
                suffix = suffix,
                onSuffixClick = { showGuide = true },
            )
        }

        item {
            ExamplesSection(examples = examples)
        }

        item {
            Column(
                modifier = Modifier.width(180.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                BasicText(
                    text = "Suffixes follow oxidation states I–VIII",
                    style = QalType.Micro.copy(
                        color = QalColors.Faint,
                        textAlign = TextAlign.Center,
                    ),
                    maxLines = 1,
                )
                BasicText(
                    text = "Guide contains the full naming rules",
                    style = QalType.Micro.copy(
                        color = QalColors.Faint.copy(alpha = 0.7f),
                        textAlign = TextAlign.Center,
                    ),
                    maxLines = 1,
                )
                Spacer(Modifier.height(20.dp))
            }
        }
    }

    if (showPicker) {
        ElementPickerDialog(
            selectedSymbol = element.symbol,
            onSelect = { state.selectElement(it) },
            onDismiss = { showPicker = false },
        )
    }

    if (showGuide) {
        GuideDialog(onDismiss = { showGuide = false })
    }
}

@Composable
private fun ElementCard(
    element: ChemElement,
    onClick: () -> Unit,
    onGuide: () -> Unit,
) {
    val haptics = LocalHapticFeedback.current
    val shape = RoundedCornerShape(18.dp)
    Row(
        modifier = Modifier
            .width(180.dp)
            .height(42.dp)
            .clip(shape)
            .background(QalColors.Card)
            .border(1.dp, QalColors.BorderSoft, shape)
            .clickable {
                haptics.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                onClick()
            }
            .padding(horizontal = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier = Modifier
                .size(32.dp)
                .clip(CircleShape)
                .background(
                    Brush.linearGradient(
                        listOf(QalColors.VioletDeep, QalColors.Violet),
                    ),
                )
                .border(1.dp, QalColors.VioletSoft.copy(alpha = 0.35f), CircleShape),
            contentAlignment = Alignment.Center,
        ) {
            BasicText(
                text = element.symbol,
                style = QalType.Numeric.copy(
                    color = QalColors.Text,
                    fontWeight = FontWeight.ExtraBold,
                    fontSize = 12.sp,
                ),
                maxLines = 1,
            )
        }
        Spacer(Modifier.width(8.dp))
        Column(modifier = Modifier.weight(1f)) {
            BasicText(
                text = element.czech,
                style = QalType.TitleSmall.copy(color = QalColors.Text),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            BasicText(
                text = "${element.english} · ${element.states.size} state${if (element.states.size == 1) "" else "s"}",
                style = QalType.Micro.copy(color = QalColors.Muted),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        QalPill(
            text = "Guide",
            onClick = onGuide,
            accent = true,
        )
        Spacer(Modifier.width(3.dp))
        GlyphIcon(Glyph.ChevronDown, QalColors.Faint, Modifier.size(11.dp))
    }
}

@Composable
private fun OxidationChips(
    element: ChemElement,
    selected: Int,
    onSelect: (Int) -> Unit,
) {
    val haptics = LocalHapticFeedback.current
    Row(
        modifier = Modifier
            .width(180.dp)
            .height(24.dp),
        horizontalArrangement = Arrangement.spacedBy(4.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        element.states.forEach { state ->
            val isSelected = state.oxidation == selected
            val background by animateColorAsState(
                targetValue = if (isSelected) QalColors.Violet else QalColors.CardLift,
                label = "stateChipBackground",
            )
            Box(
                modifier = Modifier
                    .width(30.dp)
                    .height(24.dp)
                    .clip(RoundedCornerShape(12.dp))
                    .background(background)
                    .border(
                        width = 1.dp,
                        color = if (isSelected) Color.Transparent else QalColors.BorderSoft,
                        shape = RoundedCornerShape(12.dp),
                    )
                    .clickable {
                        haptics.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                        onSelect(state.oxidation)
                    }
                    .semantics(mergeDescendants = true) {
                        contentDescription = "Oxidation state ${ChemData.roman(state.oxidation)}"
                        role = Role.Button
                    },
                contentAlignment = Alignment.Center,
            ) {
                BasicText(
                    text = ChemData.roman(state.oxidation),
                    style = QalType.Micro.copy(
                        color = if (isSelected) QalColors.Ink else QalColors.Muted,
                        fontWeight = FontWeight.Bold,
                        fontSize = 8.sp,
                    ),
                    maxLines = 1,
                )
            }
        }
    }
}

@Composable
private fun ResultCard(
    element: ChemElement,
    state: ChemState,
    suffix: String?,
    onSuffixClick: () -> Unit,
) {
    val shape = RoundedCornerShape(22.dp)
    Column(
        modifier = Modifier
            .width(180.dp)
            .clip(shape)
            .background(
                Brush.verticalGradient(
                    listOf(
                        QalColors.Violet.copy(alpha = 0.20f),
                        QalColors.Card,
                    ),
                ),
            )
            .border(1.dp, QalColors.Violet.copy(alpha = 0.26f), shape)
            .padding(10.dp),
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            FittedText(
                text = state.adjective,
                color = QalColors.Text,
                modifier = Modifier.weight(1f),
                maxSp = 22,
                minSp = 15,
                textAlign = TextAlign.Start,
            )
            if (suffix != null) {
                Spacer(Modifier.width(4.dp))
                SuffixBadge(
                    suffix = suffix,
                    onClick = onSuffixClick,
                )
            }
        }
        Spacer(Modifier.height(2.dp))
        BasicText(
            text = "${element.symbol}${ChemData.superscriptCharge(state.oxidation)} · oxidation state ${ChemData.roman(state.oxidation)}",
            style = QalType.Micro.copy(color = QalColors.Muted),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
private fun ExamplesSection(examples: List<CompoundExample>) {
    Column(
        modifier = Modifier
            .width(180.dp)
            .padding(top = 1.dp),
    ) {
        BasicText(
            text = "EXAMPLES",
            style = QalType.Label.copy(color = QalColors.Faint),
            maxLines = 1,
        )
        Spacer(Modifier.height(4.dp))
        examples.forEach { example ->
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(30.dp)
                    .clip(RoundedCornerShape(12.dp))
                    .background(QalColors.Card)
                    .border(1.dp, QalColors.BorderSoft, RoundedCornerShape(12.dp))
                    .padding(horizontal = 9.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                BasicText(
                    text = example.name,
                    modifier = Modifier.weight(1f),
                    style = QalType.Caption.copy(color = QalColors.Muted),
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Spacer(Modifier.width(6.dp))
                FormulaText(
                    formula = example.formula,
                    color = QalColors.VioletSoft,
                    fontSize = 10,
                )
            }
            Spacer(Modifier.height(4.dp))
        }
    }
}

@Composable
private fun ElementPickerDialog(
    selectedSymbol: String,
    onSelect: (String) -> Unit,
    onDismiss: () -> Unit,
) {
    val haptics = LocalHapticFeedback.current
    val rows = remember { ChemData.elements.chunked(3) }
    val selectedRow = remember(selectedSymbol) {
        ChemData.elements.indexOfFirst { it.symbol == selectedSymbol }.coerceAtLeast(0) / 3
    }

    QalDialog(onDismissRequest = onDismiss) {
        DialogHeader(
            title = "Element",
            subtitle = "Czech names",
            onClose = onDismiss,
        )
        Spacer(Modifier.height(4.dp))
        ScalingLazyColumn(
            modifier = Modifier.fillMaxSize(),
            state = rememberScalingLazyListState(initialCenterItemIndex = selectedRow),
            contentPadding = PaddingValues(horizontal = 14.dp, vertical = 6.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            items(rows) { row ->
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(5.dp),
                ) {
                    row.forEach { element ->
                        ElementTile(
                            element = element,
                            selected = element.symbol == selectedSymbol,
                            onClick = {
                                haptics.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                                onSelect(element.symbol)
                                onDismiss()
                            },
                            modifier = Modifier.weight(1f),
                        )
                    }
                    repeat(3 - row.size) {
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
private fun ElementTile(
    element: ChemElement,
    selected: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val shape = RoundedCornerShape(14.dp)
    Column(
        modifier = modifier
            .height(44.dp)
            .clip(shape)
            .background(
                if (selected) QalColors.Violet.copy(alpha = 0.16f) else QalColors.Card,
            )
            .border(
                width = 1.dp,
                color = if (selected) QalColors.Violet.copy(alpha = 0.55f) else QalColors.BorderSoft,
                shape = shape,
            )
            .clickable(onClick = onClick)
            .padding(horizontal = 4.dp, vertical = 5.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        BasicText(
            text = element.symbol,
            style = QalType.Numeric.copy(
                color = if (selected) QalColors.VioletSoft else QalColors.Text,
                fontWeight = FontWeight.ExtraBold,
                fontSize = 12.sp,
            ),
            maxLines = 1,
        )
        BasicText(
            text = element.czech,
            style = QalType.Micro.copy(
                color = QalColors.Muted,
                fontSize = 6.5.sp,
                textAlign = TextAlign.Center,
            ),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
private fun GuideDialog(onDismiss: () -> Unit) {
    QalDialog(onDismissRequest = onDismiss) {
        DialogHeader(
            title = "Guide",
            subtitle = "Czech nomenclature",
            onClose = onDismiss,
        )
        Spacer(Modifier.height(4.dp))
        ScalingLazyColumn(
            modifier = Modifier.fillMaxSize(),
            state = rememberScalingLazyListState(initialCenterItemIndex = 0),
            contentPadding = PaddingValues(horizontal = 14.dp, vertical = 6.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp),
            autoCentering = null,
            scalingParams = ScalingLazyColumnDefaults.scalingParams(
                edgeScale = 1f,
                edgeAlpha = 1f,
                minElementHeight = 1f,
                maxElementHeight = 1f,
            ),
        ) {
            item {
                BasicText(
                    text = "The ending of the Czech adjective encodes the oxidation state. Start with the suffix table, then use the same adjective in oxides, hydroxides, acids and salts.",
                    modifier = Modifier.padding(horizontal = 4.dp),
                    style = QalType.Caption.copy(color = QalColors.Muted),
                )
            }

            item {
                GuideCard(title = "Oxidation suffixes") {
                    ChemData.suffixRules.forEach { rule ->
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .height(26.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            BasicText(
                                text = ChemData.roman(rule.oxidation),
                                modifier = Modifier.width(20.dp),
                                style = QalType.Micro.copy(
                                    color = QalColors.VioletSoft,
                                    fontWeight = FontWeight.Bold,
                                ),
                                maxLines = 1,
                            )
                            BasicText(
                                text = rule.suffix,
                                modifier = Modifier.width(58.dp),
                                style = QalType.Caption.copy(
                                    color = QalColors.Text,
                                    fontWeight = FontWeight.Bold,
                                    fontSize = 8.sp,
                                ),
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                            BasicText(
                                text = rule.exampleName,
                                modifier = Modifier.weight(1f),
                                style = QalType.Micro.copy(color = QalColors.Muted),
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                            FormulaText(
                                formula = rule.exampleFormula,
                                fontSize = 8,
                            )
                        }
                    }
                    BasicText(
                        text = "-ečný is used for fosforečný, chlorečný and bismutečný; other elements use -ičný.",
                        style = QalType.Micro.copy(color = QalColors.AmberSoft),
                    )
                }
            }

            items(ChemData.guideSections) { section ->
                GuideCard(title = section.title) {
                    BasicText(
                        text = section.body,
                        style = QalType.Caption.copy(color = QalColors.Muted),
                    )
                    section.steps.forEachIndexed { index, step ->
                        BasicText(
                            text = "${index + 1}. $step",
                            modifier = Modifier.padding(top = 3.dp),
                            style = QalType.Micro.copy(color = QalColors.Text),
                        )
                    }
                    if (section.examples.isNotEmpty()) {
                        Spacer(Modifier.height(4.dp))
                    }
                    section.examples.forEach { example ->
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(top = 2.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            BasicText(
                                text = example.name,
                                modifier = Modifier.weight(1f),
                                style = QalType.Micro.copy(color = QalColors.Muted),
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                            Spacer(Modifier.width(6.dp))
                            FormulaText(
                                formula = example.formula,
                                fontSize = 8,
                            )
                        }
                    }
                }
            }

            item {
                BasicText(
                    text = "Reference: Czech grammar-school nomenclature (IUPAC principles).",
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 6.dp, vertical = 4.dp),
                    style = QalType.Micro.copy(
                        color = QalColors.Faint,
                        textAlign = TextAlign.Center,
                    ),
                )
                Spacer(Modifier.height(24.dp))
            }
        }
    }
}

@Composable
private fun GuideCard(
    title: String,
    content: @Composable ColumnScope.() -> Unit,
) {
    val shape = RoundedCornerShape(18.dp)
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(shape)
            .background(QalColors.Card)
            .border(1.dp, QalColors.BorderSoft, shape)
            .padding(10.dp),
    ) {
        BasicText(
            text = title,
            style = QalType.TitleSmall.copy(color = QalColors.Text),
            maxLines = 1,
        )
        Spacer(Modifier.height(4.dp))
        content()
    }
}
