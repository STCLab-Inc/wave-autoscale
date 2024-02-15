'use client';

import dynamic from 'next/dynamic';
import { useEffect, useState } from 'react';
import { deserializeMetricDefinitions } from '@/utils/metric-binding';
import { MetricDefinition } from '@/types/bindings/metric-definition';
import { debounce } from 'lodash';
import { renderKeyValuePairsWithJson } from '../common/keyvalue-renderer';
import EnabledBadge from '../common/enabled-badge';
import WASimpleTable from '../common/wa-simple-table';
import { PageSectionTitle } from '../common/page-section-title';
import MetricService from '@/services/metric';
import PageHeader from '../common/page-header';
import { createColumnHelper } from '@tanstack/react-table';
import { errorToast, successToast } from '@/utils/toast';
import { getAnnotationsFromError } from '../common/yaml-editor/annotation';

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
async function getMetricYamls(): Promise<string[]> {
  const yamls = await MetricService.getMetricYamls();
  return yamls;
}

// Page
export default function MetricsPage() {
  const [yaml, setYaml] = useState<string>('');
  const [previewData, setPreviewData] = useState<MetricDefinition[]>([]);
  const [annotations, setAnnotations] = useState<any[]>([]);
  // Effects
  useEffect(() => {
    loadFromService();
  }, []);

  // Handlers

  // Load from service
  const loadFromService = async () => {
    setAnnotations([]);

    try {
      const yamls = await getMetricYamls();
      const mergedYaml = yamls.join('---\n');
      handleYamlChange(mergedYaml);
    } catch (error: any) {
      console.error(error);
      errorToast(error.message);
      return;
    }
  };

  // YAML change
  const handleYamlChange = debounce((value: string) => {
    setAnnotations([]);
    setYaml(value);

    let newPreviewData;
    try {
      newPreviewData = deserializeMetricDefinitions(value);
    } catch (error: any) {
      console.log(error);
      const annotations = getAnnotationsFromError(error);
      setAnnotations(annotations);
      return;
    }

    try {
      setPreviewData(newPreviewData);
    } catch (error: any) {
      console.error(error);
      errorToast(error.message);
      return;
    }
  }, 500);

  const handleReset = () => {
    handleYamlChange.cancel();
    loadFromService();
  };

  const handleSave = async () => {
    handleYamlChange.flush();
    try {
      const result = await MetricService.syncMetricYaml(yaml);
      console.log(result);
    } catch (error: any) {
      console.error(error);
      errorToast(error.message);
      return;
    }
    loadFromService();
    successToast('Saved');
  };

  return (
    <main className="flex h-full w-full flex-col">
      {/* Header */}
      <PageHeader title="Metric Definitions" />
      {/* Sections */}
      <div className="min-height-0 flex w-full flex-1 space-x-8">
        {/* Preview */}
        <div className="flex h-full flex-1 flex-col overflow-y-auto">
          {/* Preview Title */}
          <div className="flex h-14 items-center px-6">
            <PageSectionTitle title="Preview" />
          </div>
          {/* Table */}
          <div className="px-6 pb-6">
            <WASimpleTable<MetricDefinition>
              tableOptions={{
                data: previewData,
                columns,
              }}
            />
          </div>
        </div>
        {/* Code */}
        <div className="flex h-full flex-1 flex-col bg-wa-gray-50 shadow-[-4px_0px_8px_rgba(23,25,28,0.08)]">
          {/* Code Title */}
          <div className="border-wa-gray-700 flex h-14 items-center border-b px-6">
            <div className="flex-1">
              <PageSectionTitle title="Code" />
            </div>
            <div className="flex items-center space-x-4">
              <button
                className="btn-ghost btn-sm btn flex h-8 items-center justify-center rounded-md text-sm"
                onClick={handleReset}
              >
                Reset Code
              </button>
              <button
                className="btn-gray btn-sm btn flex h-8 items-center justify-center rounded-md text-sm"
                onClick={handleSave}
                disabled={annotations.length > 0}
              >
                Save
              </button>
            </div>
          </div>
          <YAMLEditor
            value={yaml}
            onChange={handleYamlChange}
            annotations={annotations}
          />
        </div>
      </div>
    </main>
  );
}
