'use client';

import React, { useEffect, useState } from 'react';

import ScalingPlanService from '@/services/scaling-plan';
import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';

import ScalingPlansSidebar from './scaling-plans-sidebar';
import ScalingPlansTabs from './scaling-plans-tabs';
import ScalingPlanDiagramComponent from './diagram/scaling-plans-diagram';
import ScalingPlanCodeComponent from './code/scaling-plans-code';
import ScalingPlansDrawer from './scaling-plans-drawer';
import ContentHeader from '../common/content-header';

async function getScalingPlans() {
  const scalingPlans = await ScalingPlanService.getScalingPlans();
  return scalingPlans;
}

interface ScalingPlanDefinitionEx extends ScalingPlanDefinition {
  metadata: { cool_down: number; interval: number; title: string };
}

export default function ScalingPlansPage() {
  const [scalingPlans, setScalingPlans] = useState<ScalingPlanDefinitionEx[]>(
    []
  );
  const [scalingPlansItem, setScalingPlansItem] =
    useState<ScalingPlanDefinitionEx>();

  const fetchScalingPlans = async () => {
    try {
      const db_id = scalingPlansItem?.db_id;
      let scalingPlans = await getScalingPlans();
      setScalingPlans(scalingPlans);
      setScalingPlansItem(
        scalingPlans.find(
          (item: ScalingPlanDefinitionEx) => item.db_id === db_id
        )
      );
    } catch (error) {
      console.error(error);
      return [];
    }
  };

  useEffect(() => {
    fetchScalingPlans();
  }, []);

  const [detailsModalFlag, setDetailsModalFlag] = useState(false);

  const [fetchFlag, setFetchFlag] = useState(false);

  useEffect(() => {
    if (fetchFlag) {
      fetchScalingPlans();
      setFetchFlag(false);
    }
  }, [fetchFlag]);

  const [viewMode, setViewMode] = useState('diagram');

  return (
    <main className="flex h-full w-full flex-row">
      <ScalingPlansSidebar
        scalingPlans={scalingPlans}
        scalingPlansItem={scalingPlansItem}
        setScalingPlansItem={setScalingPlansItem}
        detailsModalFlag={detailsModalFlag}
        setDetailsModalFlag={setDetailsModalFlag}
        setFetchFlag={setFetchFlag}
      />
      <div className="flex h-full w-full flex-col">
        {scalingPlansItem ? (
          <div className="flex h-full w-full flex-col">
            <ContentHeader type="SUB" title={scalingPlansItem.id} />
            <ScalingPlansTabs viewMode={viewMode} setViewMode={setViewMode} />
            {viewMode === 'diagram' ? (
              <ScalingPlanDiagramComponent
                scalingPlansItem={scalingPlansItem}
                setScalingPlansItem={setScalingPlansItem}
                detailsModalFlag={detailsModalFlag}
                setDetailsModalFlag={setDetailsModalFlag}
                setFetchFlag={setFetchFlag}
              />
            ) : viewMode === 'code' ? (
              <ScalingPlanCodeComponent
                scalingPlansItem={scalingPlansItem}
                setScalingPlansItem={setScalingPlansItem}
                detailsModalFlag={detailsModalFlag}
                setDetailsModalFlag={setDetailsModalFlag}
                setFetchFlag={setFetchFlag}
              />
            ) : null}
          </div>
        ) : (
          <div className="flex h-full w-full flex-col border-t "></div>
        )}
      </div>
      {detailsModalFlag ? (
        <ScalingPlansDrawer
          scalingPlansItem={scalingPlansItem}
          setScalingPlansItem={setScalingPlansItem}
          setDetailsModalFlag={setDetailsModalFlag}
          setFetchFlag={setFetchFlag}
        />
      ) : null}
    </main>
  );
}
