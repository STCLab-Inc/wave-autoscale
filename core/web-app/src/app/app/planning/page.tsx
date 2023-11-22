'use client';

import React, { useEffect, useState } from 'react';

import PlanService from '@/services/plan';
import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';

import PlanningSidebar from './sidebar';
import SubContentHeader from '../subcontent-header';
import PlanningTabs from './planning-tabs';
import PlanningDiagramComponent from './diagram/planning-diagram';
import PlanningCodeComponent from './code/planning-code';

import PlanningDrawer from './planning-drawer';

async function getPlans() {
  const plans = await PlanService.getPlans();
  return plans;
}

interface ScalingPlanDefinitionEx extends ScalingPlanDefinition {
  metadata: { cool_down: number; interval: number; title: string };
}

export default function PlanningPage() {
  const [plans, setPlans] = useState<ScalingPlanDefinitionEx[]>([]);
  const [plansItem, setPlansItem] = useState<ScalingPlanDefinitionEx>();

  const fetchPlans = async () => {
    try {
      const db_id = plansItem?.db_id;
      let plans = await getPlans();
      setPlans(plans);
      setPlansItem(
        plans.find((item: ScalingPlanDefinitionEx) => item.db_id === db_id)
      );
    } catch (error) {
      console.error(error);
      return [];
    }
  };

  useEffect(() => {
    fetchPlans();
  }, []);

  const [detailsModalFlag, setDetailsModalFlag] = useState(false);

  const [fetchFlag, setFetchFlag] = useState(false);

  useEffect(() => {
    if (fetchFlag) {
      fetchPlans();
      setFetchFlag(false);
    }
  }, [fetchFlag]);

  const [viewMode, setViewMode] = useState('diagram');

  return (
    <main className="flex h-full w-full flex-row">
      <PlanningSidebar
        plans={plans}
        plansItem={plansItem}
        setPlansItem={setPlansItem}
        detailsModalFlag={detailsModalFlag}
        setDetailsModalFlag={setDetailsModalFlag}
        setFetchFlag={setFetchFlag}
      />
      <div className="flex h-full w-full flex-col">
        {plansItem ? (
          <div className="flex h-full w-full flex-col">
            <SubContentHeader title={plansItem.id} />
            <PlanningTabs viewMode={viewMode} setViewMode={setViewMode} />
            {viewMode === 'diagram' ? (
              <PlanningDiagramComponent
                plansItem={plansItem}
                setPlansItem={setPlansItem}
                detailsModalFlag={detailsModalFlag}
                setDetailsModalFlag={setDetailsModalFlag}
                setFetchFlag={setFetchFlag}
              />
            ) : viewMode === 'code' ? (
              <PlanningCodeComponent
                plansItem={plansItem}
                setPlansItem={setPlansItem}
                detailsModalFlag={detailsModalFlag}
                setDetailsModalFlag={setDetailsModalFlag}
                setFetchFlag={setFetchFlag}
              />
            ) : null}
          </div>
        ) : null}
      </div>
      {detailsModalFlag ? (
        <PlanningDrawer
          plansItem={plansItem}
          setPlansItem={setPlansItem}
          setDetailsModalFlag={setDetailsModalFlag}
          setFetchFlag={setFetchFlag}
        />
      ) : null}
    </main>
  );
}
