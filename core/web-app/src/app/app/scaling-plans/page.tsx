'use client';

import React, { useEffect, useMemo, useState } from 'react';
import ScalingPlanService from '@/services/scaling-plan';
import ScalingPlansSidebar from './scaling-plans-sidebar';
import ScalingPlanDiagram from './scaling-plan-diagram';
import { PageSectionTitle } from '../common/page-section-title';
import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';
import {
  deserializeScalingPlanDefinition,
  serializeScalingPlanDefinition,
} from '@/utils/scaling-plan-binding';
import { debounce } from 'lodash';
import dynamic from 'next/dynamic';
import PageHeader from '../common/page-header';
import DefinitionCountBadge from '../common/definition-count-badge';
import { errorToast, successToast } from '@/utils/toast';

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
  const [annotations, setAnnotations] = useState<any[]>([]);

  // Show preview from YAML automatically
  const selectedPlanPreview = useMemo<ScalingPlanDefinition | undefined>(() => {
    setAnnotations([]);
    if (selectedPlanIndex === undefined) {
      return;
    }
    try {
      const deserializedPlan =
        deserializeScalingPlanDefinition(selectedPlanYaml);
      return deserializedPlan;
    } catch (error: any) {
      console.log(error);
      // TODO: Annotations
      // const annotations = getAnnotationsFromError(error);
      // setAnnotations(annotations);
    }
    return undefined;
  }, [selectedPlanYaml, selectedPlanIndex]);

  // Effects
  useEffect(() => {
    fetch();
  }, []);

  useEffect(() => {
    if (scalingPlans.length > 0 && selectedPlanIndex === undefined) {
      handleSelectedPlan(0);
    }
  }, [scalingPlans, selectedPlanIndex]);

  // Handlers
  const fetch = async () => {
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
    setAnnotations([]);
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
  }, 500);

  const handleDelete = async () => {
    // Confirm
    const confirmed = confirm('Are you sure to delete this plan?');
    if (!confirmed) {
      return;
    }
    // Delete
    if (selectedPlanIndex === undefined) {
      return;
    }
    const plan = scalingPlans[selectedPlanIndex];
    await ScalingPlanService.deleteScalingPlan(plan.db_id);
    setSelectedPlanIndex(undefined);
    fetch();
    successToast('Deleted!');
  };

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
      if (!scalingPlan?.id) {
        handleDelete();
        return;
      }
      const result = await ScalingPlanService.createScalingPlan(scalingPlan);
      console.info({ result });
      fetch();
      successToast('Saved!');
    } catch (error: any) {
      console.error(error);
      errorToast(error.message);
      return;
    }
  };

  return (
    <main className="flex h-full w-full flex-col">
      {/* Header */}
      <PageHeader title="Scaling Plans" />
      {/* Sections */}
      <div className="min-height-0 flex w-full flex-1 ">
        {/* Plans */}
        <div className="flex h-full w-72 flex-col overflow-y-auto border-r bg-wa-gray-50">
          {/* Plans Header */}
          <div className="flex h-14 items-center border-b border-wa-gray-200 pl-6">
            <PageSectionTitle title="Plans" />
            <DefinitionCountBadge count={scalingPlans.length} />
            <button
              className="btn-image btn flex h-8"
              onClick={handleAddScalingPlan}
            >
              <img src="/assets/scaling-plans/add.svg" alt="plus" />
            </button>
          </div>
          <ScalingPlansSidebar
            scalingPlans={scalingPlans}
            selectedIndex={selectedPlanIndex}
            onChange={handleSelectedPlan}
          />
        </div>
        {/* Diagram */}
        <div className="relative flex h-full flex-1 flex-col overflow-y-auto">
          <div className="fixed z-10 flex h-14 items-center px-6">
            <PageSectionTitle title="Diagram" />
          </div>
          <ScalingPlanDiagram scalingPlan={selectedPlanPreview} />
        </div>
        {/* Code */}
        <div className="flex h-full flex-1 flex-col bg-wa-gray-50 shadow-[-4px_0px_8px_rgba(23,25,28,0.08)]">
          {/* Code Title */}
          <div className="border-wa-gray-700 flex h-14 items-center border-b px-6">
            <div className="flex-1">
              <PageSectionTitle title="Code" />
            </div>
            {/* Code Controls */}
            <div className="flex items-center">
              {/* Delete button */}
              <button
                className="btn-image btn mx-0 flex h-8 px-0"
                onClick={handleDelete}
              >
                <img src="/assets/scaling-plans/delete.svg" alt="delete" />
              </button>
              {/* divider */}
              <div className="mx-5 h-6 w-[1px] bg-wa-gray-400" />
              <button
                className="btn-ghost btn-sm btn mr-5 flex h-8 items-center justify-center rounded-md text-sm"
                onClick={handleReset}
              >
                Reset
              </button>
              <button
                className="btn-gray btn-sm btn flex h-8 items-center justify-center rounded-md text-sm"
                onClick={handleSave}
              >
                Save
              </button>
            </div>
          </div>
          <YAMLEditor
            value={selectedPlanYaml}
            onChange={handleYamlChange}
            annotations={annotations}
          />
        </div>
      </div>
    </main>
  );
}
