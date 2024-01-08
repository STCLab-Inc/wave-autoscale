'use client';

import React, { useEffect, useState } from 'react';
import ScalingPlanService from '@/services/scaling-plan';
import ScalingPlansSidebar from './scaling-plans-sidebar';
import ScalingPlansTabs from './scaling-plans-tabs';
import ScalingPlanDiagram from './scaling-plan-diagram';
import ScalingPlanCode from './scaling-plan-code';
import ScalingPlanDrawer from './scaling-plan-drawer';
import ContentHeader from '../common/content-header';
import { ScalingPlanDefinitionEx } from './scaling-plan-definition-ex';

enum ViewMode {
  DIAGRAM = 'diagram',
  CODE = 'code',
}

const TAB_ITEMS: { viewMode: ViewMode; label: string }[] = [
  { viewMode: ViewMode.DIAGRAM, label: 'Diagram' },
  { viewMode: ViewMode.CODE, label: 'Code' },
];

// Utils
async function getScalingPlans(): Promise<ScalingPlanDefinitionEx[]> {
  const scalingPlans = await ScalingPlanService.getScalingPlans();
  return scalingPlans;
}

export default function ScalingPlansPage() {
  const [scalingPlans, setScalingPlans] = useState<ScalingPlanDefinitionEx[]>(
    []
  );
  const [selectedScalingPlanDbId, setSelectedScalingPlanDbId] =
    useState<string>();
  const [drawerVisible, setDrawerVisible] = useState<boolean>(false);

  // UI states
  const [detailsModalFlag, setDetailsModalFlag] = useState(false);
  const [forceFetch, setForceFetch] = useState(false);
  const [viewMode, setViewMode] = useState<ViewMode>(ViewMode.DIAGRAM);

  // Effects
  useEffect(() => {
    fetchScalingPlans();
  }, []);

  useEffect(() => {
    if (forceFetch) {
      setForceFetch(false);
      fetchScalingPlans();
    }
  }, [forceFetch]);

  // Handlers
  const fetchScalingPlans = async () => {
    try {
      let scalingPlans = await getScalingPlans();
      setScalingPlans(scalingPlans);
      // Unselect scaling plan if it is not in the list
      let selectedScalingPlan = scalingPlans.find(
        (scalingPlan) => scalingPlan.db_id === selectedScalingPlanDbId
      );
      if (!selectedScalingPlan) {
        setSelectedScalingPlanDbId(undefined);
      }
    } catch (error) {
      console.error(error);
    }
  };

  const handleAddScalingPlan = () => {
    setSelectedScalingPlanDbId(undefined);
    setDetailsModalFlag(true);
  };

  // Set selected scaling plan
  const selectedScalingPlan = scalingPlans.find(
    (scalingPlan) => scalingPlan.db_id === selectedScalingPlanDbId
  );

  return (
    <main className="flex h-full w-full flex-row">
      <div className="flex h-full">
        <aside className="flex h-full w-96 flex-col border-r border-gray-200">
          <div className="flex h-14 min-h-14 w-full min-w-full flex-row items-center justify-between border-b border-t border-gray-200 bg-gray-75 px-8">
            <span className="font-Pretendard whitespace-nowrap text-lg font-semibold text-gray-1000">
              Scaling Plans ({scalingPlans.length})
            </span>
            <div className="flex items-center">
              <button
                className="flex h-8 items-center justify-center rounded-md border border-blue-400 bg-blue-400  pl-5 pr-5 text-sm text-gray-50"
                onClick={() => {
                  setSelectedScalingPlanDbId(undefined);
                  setDrawerVisible(true);
                }}
                type="button"
              >
                ADD PLAN
              </button>
            </div>
          </div>
          <ScalingPlansSidebar
            scalingPlans={scalingPlans}
            selectedScalingPlanDbId={selectedScalingPlanDbId}
            onChange={setSelectedScalingPlanDbId}
          />
        </aside>
      </div>
      <div className="flex h-full w-full flex-col">
        {selectedScalingPlan ? (
          <div className="flex h-full w-full flex-col">
            <ContentHeader type="INNER" title={selectedScalingPlan.id} />
            <ScalingPlansTabs<ViewMode>
              tabItems={TAB_ITEMS}
              selected={viewMode}
              onChange={(viewMode) => setViewMode(viewMode)}
            />
            {viewMode === ViewMode.DIAGRAM ? (
              <ScalingPlanDiagram scalingPlan={selectedScalingPlan} />
            ) : viewMode === ViewMode.CODE ? (
              <ScalingPlanCode
                scalingPlan={selectedScalingPlan}
                onChange={() => setForceFetch(true)}
              />
            ) : null}
          </div>
        ) : (
          <div className="flex h-full w-full flex-col border-t "></div>
        )}
      </div>
      {drawerVisible ? (
        <ScalingPlanDrawer
          scalingPlan={selectedScalingPlan}
          onChange={() => setForceFetch(true)}
          onClose={() => setDrawerVisible(false)}
        />
      ) : null}
    </main>
  );
}
