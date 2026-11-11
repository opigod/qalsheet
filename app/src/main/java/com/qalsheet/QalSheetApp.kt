package com.qalsheet

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.wear.compose.foundation.hierarchicalFocusGroup
import androidx.wear.compose.foundation.pager.HorizontalPager
import androidx.wear.compose.foundation.pager.rememberPagerState
import androidx.wear.compose.foundation.rotary.RotaryScrollableDefaults
import com.qalsheet.calc.CalculatorPage
import com.qalsheet.calc.CalculatorState
import com.qalsheet.chem.ChemistryPage
import com.qalsheet.chem.ChemistryState
import com.qalsheet.convert.ConverterPage
import com.qalsheet.convert.ConverterState
import com.qalsheet.ui.QalScreen
import com.qalsheet.ui.QalTabs
import kotlinx.coroutines.launch

@Composable
fun QalSheetApp() {
    val pagerState = rememberPagerState(pageCount = { 3 })
    val scope = rememberCoroutineScope()

    val calculatorState = remember { CalculatorState() }
    val converterState = remember { ConverterState() }
    val chemistryState = remember { ChemistryState() }

    QalScreen {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(top = 12.dp, bottom = 14.dp)
                .hierarchicalFocusGroup(active = true),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            QalTabs(
                selectedPage = pagerState.currentPage,
                onSelect = { page ->
                    scope.launch {
                        pagerState.animateScrollToPage(page)
                    }
                },
            )
            Spacer(Modifier.height(2.dp))
            HorizontalPager(
                modifier = Modifier
                    .fillMaxSize()
                    .weight(1f),
                state = pagerState,
                rotaryScrollableBehavior = RotaryScrollableDefaults.snapBehavior(pagerState),
            ) { page ->
                when (page) {
                    0 -> CalculatorPage(
                        state = calculatorState,
                        modifier = Modifier.fillMaxSize(),
                    )
                    1 -> ConverterPage(
                        state = converterState,
                        modifier = Modifier.fillMaxSize(),
                    )
                    else -> ChemistryPage(
                        state = chemistryState,
                        modifier = Modifier.fillMaxSize(),
                    )
                }
            }
        }
    }
}
