'use client';

import ContentHeader from '../common/content-header';
import dynamic from 'next/dynamic';
import MetricService from '@/services/metric';
import { useEffect, useState } from 'react';
import {
  deserializeMetricDefinitions,
  serializeMetricDefinitions,
} from '@/utils/metric-binding';
import { MetricDefinition } from '@/types/bindings/metric-definition';
import { debounce } from 'lodash';
import { createColumnHelper } from '@tanstack/react-table';
import { renderKeyValuePairsWithJson } from '../common/keyvalue-renderer';
import EnabledBadge from '../common/enabled-badge';
import WASimpleTable from '../common/wa-simple-table';

// Dynamic imports (because of 'window' object)
const YAMLEditor = dynamic(() => import('../common/yaml-editor'), {
  ssr: false,
});

// Table columns
const columnHelper = createColumnHelper<MetricDefinition>();
const columns = [
  columnHelper.accessor('id', {
    header: () => 'ID',
    cell: (cell) => cell.getValue(),
  }),
  columnHelper.accessor('collector', {
    header: () => 'Collector',
    cell: (cell) => cell.getValue(),
  }),
  columnHelper.accessor('metadata', {
    header: () => 'Metadata',
    cell: (cell) => (
      <span className="flex flex-col items-center break-all text-start">
        {renderKeyValuePairsWithJson(cell.getValue(), true)}
      </span>
    ),
  }),
  columnHelper.accessor('enabled', {
    header: () => 'Enabled',
    cell: (cell) => <EnabledBadge enabled={cell.getValue()} />,
  }),
];

// Service
async function getMetrics(): Promise<MetricDefinition[]> {
  const metrics = await MetricService.getMetrics();
  return metrics;
}

// Page
export default function MetricsPage() {
  const [yaml, setYaml] = useState<string>('');
  const [previewData, setPreviewData] = useState<MetricDefinition[]>([]);

  // Effects
  useEffect(() => {
    loadFromService();
  }, []);

  // Handlers
  const loadFromService = async () => {
    const metrics = await getMetrics();
    // YAML
    try {
      const newYaml = serializeMetricDefinitions(metrics);
      setYaml(newYaml);
    } catch (error: any) {
      console.error(error);
      alert(error.message);
      return;
    }

    // Preview
    setPreviewData(metrics);
  };

  const handleYamlChange = debounce((value: string) => {
    setYaml(value);

    let newPreviewData;
    try {
      newPreviewData = deserializeMetricDefinitions(value);
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
      const metricDefinitions = deserializeMetricDefinitions(yaml);
      const promises = metricDefinitions.map((metricDefinition) => {
        return MetricService.createMetric(metricDefinition);
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
          title="Metrics"
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
          <WASimpleTable<MetricDefinition>
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

//
// Components
//
function SectionTitle({ title }: { title: string }) {
  return (
    <div className="mb-4 flex h-10 w-full items-center justify-start">
      <div className="text-lg font-bold">{title}</div>
    </div>
  );
}
