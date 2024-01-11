'use client';

import React, { useEffect, useMemo, useState } from 'react';
import ScalingPlanService from '@/services/scaling-plan';
import ScalingPlansSidebar from './scaling-plans-sidebar';
import ScalingPlanDiagram from './scaling-plan-diagram';
import ContentHeader from '../common/content-header';
import { SectionTitle } from '../common/section-title';
import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';
import {
  deserializeScalingPlanDefinition,
  serializeScalingPlanDefinition,
} from '@/utils/scaling-plan-binding';
import { debounce } from 'lodash';
import dynamic from 'next/dynamic';

// Dynamic imports (because of 'window' object)
const YAMLEditor = dynamic(() => import('../common/yaml-editor'), {
  ssr: false,
});

// Service
async function getScalingPlans(): Promise<ScalingPlanDefinition[]> {
  const scalingPlans = await ScalingPlanService.getScalingPlans();
  return scalingPlans;
}

export default function ScalingPlansPage() {
  const [scalingPlans, setScalingPlans] = useState<ScalingPlanDefinition[]>([]);
  const [selectedPlanIndex, setSelectedPlanIndex] = useState<number>();
  const [selectedPlanYaml, setSelectedPlanYaml] = useState<string>('');
  const selectedPlanPreview = useMemo<ScalingPlanDefinition | undefined>(() => {
    if (selectedPlanIndex === undefined) {
      return;
    }
    try {
      const deserializedPlan =
        deserializeScalingPlanDefinition(selectedPlanYaml);
      return deserializedPlan;
    } catch (error: any) {
      console.error(error);
      alert(error.message);
    }
    return undefined;
  }, [selectedPlanYaml]);
  // const [selectedPlanPreview, setSelectedPlanPreview] =
  //   useState<ScalingPlanDefinition>();

  // Effects
  useEffect(() => {
    loadFromService();
  }, []);

  // Handlers
  const loadFromService = async () => {
    const newScalingPlans = await getScalingPlans();
    setScalingPlans(newScalingPlans);
  };

  const handleAddScalingPlan = async () => {
    // Generate new ID that is not used
    let newId = 'new_plan_';
    let numbering = 1;
    while (scalingPlans.find((p) => p.id === `${newId}${numbering}`)) {
      numbering++;
    }
    newId = `${newId}${numbering}`;
    const newScalingPlan = {
      id: newId,
    } as ScalingPlanDefinition;

    // Add new scaling plan
    setScalingPlans([...scalingPlans, newScalingPlan]);
  };

  const handleSelectedPlan = (index: number | undefined) => {
    setSelectedPlanIndex(index);
    if (index === undefined) {
      setSelectedPlanYaml('');
      return;
    }

    const yaml = serializeScalingPlanDefinition(scalingPlans[index]);
    setSelectedPlanYaml(yaml);
  };

  const handleYamlChange = debounce((value: string) => {
    setSelectedPlanYaml(value);
  }, 1000);

  const handleReset = () => {
    if (selectedPlanIndex === undefined) {
      return;
    }
    const originalYaml = serializeScalingPlanDefinition(
      scalingPlans[selectedPlanIndex]
    );
    setSelectedPlanYaml(originalYaml);
  };

  const handleSave = async () => {
    try {
      const scalingPlan = deserializeScalingPlanDefinition(selectedPlanYaml);
      const result = await ScalingPlanService.createScalingPlan(scalingPlan);
      console.info({ result });
      loadFromService();
    } catch (error: any) {
      console.error(error);
      alert(error.message);
      return;
    }
  };

  return (
    <main className="flex h-full w-full flex-col">
      {/* Header */}
      <div className="w-full">
        <ContentHeader
          type="OUTER"
          title="Scaling Plans"
          right={<div className="flex items-center space-x-4"></div>}
        />
      </div>
      {/* Sections */}
      <div className="min-height-0 flex w-full flex-1 ">
        <div className="flex h-full w-[400px] flex-col overflow-y-auto border-r px-4 py-2">
          <div className="mb-4 flex items-center">
            <SectionTitle title="Plans" />
            <button
              className="flex h-8 w-[120px] items-center justify-center rounded-md border border-blue-400 bg-blue-400 pl-2 pr-2 text-sm text-gray-50"
              onClick={handleAddScalingPlan}
              type="button"
            >
              ADD PLAN
            </button>
          </div>
          <ScalingPlansSidebar
            scalingPlans={scalingPlans}
            selectedIndex={selectedPlanIndex}
            onChange={handleSelectedPlan}
          />
        </div>
        <div className="flex h-full flex-1 flex-col overflow-y-auto px-4 py-2">
          <SectionTitle title="Diagram" />
          <ScalingPlanDiagram scalingPlan={selectedPlanPreview} />
        </div>
        <div className="flex h-full flex-1 flex-col px-4 py-2">
          <div className="mb-4 flex items-center">
            <SectionTitle title="Code" />
            <div className="flex items-center space-x-4">
              <button
                className="flex h-8 items-center justify-center rounded-md border border-blue-400  pl-5 pr-5 text-sm text-blue-400"
                onClick={handleReset}
              >
                Reset
              </button>
              <button
                className="flex h-8 items-center justify-center rounded-md border border-blue-400 bg-blue-400  pl-5 pr-5 text-sm text-gray-50"
                onClick={handleSave}
              >
                Save
              </button>
            </div>
          </div>
          <YAMLEditor value={selectedPlanYaml} onChange={handleYamlChange} />
        </div>
      </div>
    </main>
  );
}
