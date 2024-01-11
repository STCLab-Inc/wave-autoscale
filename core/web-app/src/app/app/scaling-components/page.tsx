'use client';

import ContentHeader from '../common/content-header';
import dynamic from 'next/dynamic';
import { useEffect, useState } from 'react';
import { debounce } from 'lodash';
import { createColumnHelper } from '@tanstack/react-table';
import { renderKeyValuePairsWithJson } from '../common/keyvalue-renderer';
import EnabledBadge from '../common/enabled-badge';
import WASimpleTable from '../common/wa-simple-table';
import { ScalingComponentDefinition } from '@/types/bindings/scaling-component-definition';
import ScalingComponentService from '@/services/scaling-component';
import {
  deserializeScalingComponentDefinitions,
  serializeScalingComponentDefinitions,
} from '@/utils/scaling-component-binding';
import { SectionTitle } from '../common/section-title';

// Dynamic imports (because of 'window' object)
const YAMLEditor = dynamic(() => import('../common/yaml-editor'), {
  ssr: false,
});

// Table columns
const columnHelper = createColumnHelper<ScalingComponentDefinition>();
const columns = [
  columnHelper.accessor('id', {
    header: () => 'ID',
    cell: (cell) => cell?.getValue(),
  }),
  columnHelper.accessor('component_kind', {
    header: () => 'Component Kind',
    cell: (cell) => cell?.getValue(),
  }),
  columnHelper.accessor('metadata', {
    header: () => 'Metadata',
    cell: (cell) => (
      <span className="flex flex-col items-center break-all text-start">
        {renderKeyValuePairsWithJson(cell?.getValue(), true)}
      </span>
    ),
  }),
  columnHelper.accessor('enabled', {
    header: () => 'Enabled',
    cell: (cell) => <EnabledBadge enabled={cell?.getValue()} />,
  }),
];

// Service
async function getScalingComponents(): Promise<ScalingComponentDefinition[]> {
  const scalingComponents =
    await ScalingComponentService.getScalingComponents();
  return scalingComponents;
}

// Page
export default function ScalingComponentsPage() {
  const [yaml, setYaml] = useState<string>('');
  const [previewData, setPreviewData] = useState<ScalingComponentDefinition[]>(
    []
  );

  // Effects
  useEffect(() => {
    loadFromService();
  }, []);

  // Handlers
  const loadFromService = async () => {
    const scalingComponents = await getScalingComponents();
    // YAML
    try {
      const newYaml = serializeScalingComponentDefinitions(scalingComponents);
      setYaml(newYaml);
    } catch (error: any) {
      console.error(error);
      alert(error.message);
      return;
    }

    // Preview
    setPreviewData(scalingComponents);
  };

  const handleYamlChange = debounce((value: string) => {
    setYaml(value);

    let newPreviewData;
    try {
      newPreviewData = deserializeScalingComponentDefinitions(value);
    } catch (error: any) {
      console.error(error);
      alert(error.message);
      // TODO: Set annotations
      return;
    }

    try {
      setPreviewData(newPreviewData);
    } catch (error: any) {
      console.error(error);
      alert(error.message);
      return;
    }
  }, 1000);

  const handleReset = () => {
    handleYamlChange.cancel();
    loadFromService();
  };

  const handleSave = async () => {
    try {
      const scalingComponents = deserializeScalingComponentDefinitions(yaml);
      const promises = scalingComponents.map((scalingComponentDefinition) => {
        return ScalingComponentService.createScalingComponent(
          scalingComponentDefinition
        );
      });
      await Promise.all(promises);
    } catch (error: any) {
      console.error(error);
      alert(error.message);
      return;
    }
    loadFromService();
    alert('Saved');
  };

  return (
    <main className="flex h-full w-full flex-col">
      {/* Header */}
      <div className="w-full">
        <ContentHeader
          type="OUTER"
          title="Scaling Components"
          right={
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
          }
        />
      </div>
      {/* Sections */}
      <div className="min-height-0 flex w-full flex-1 space-x-8 p-8">
        {/* Preview */}
        <div className="flex h-full flex-1 flex-col overflow-y-auto">
          {/* Preview Title */}
          <SectionTitle title="Preview" />
          {/* Table */}
          <WASimpleTable<ScalingComponentDefinition>
            tableOptions={{
              data: previewData,
              columns,
            }}
          />
        </div>
        {/* Code */}
        <div className="flex h-full flex-1 flex-col">
          {/* Code Title */}
          <SectionTitle title="Code" />
          {/* Code Editor */}
          <YAMLEditor value={yaml} onChange={handleYamlChange} />
        </div>
      </div>
    </main>
  );
}
