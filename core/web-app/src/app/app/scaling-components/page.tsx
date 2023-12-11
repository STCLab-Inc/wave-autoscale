'use client';

import React, { useEffect, useMemo, useState } from 'react';
import { useSearchParams, useRouter } from 'next/navigation';
import ScalingComponentService from '@/services/scaling-component';
import ContentHeader from '../common/content-header';
import WATable from '../common/wa-table';
import { createColumnHelper } from '@tanstack/react-table';
import { renderKeyValuePairsWithJson } from '../common/keyvalue-renderer';
import Pagination from '@/utils/pagination';
import dayjs from 'dayjs';
import { ScalingComponentDefinitionEx } from './scaling-component-definition-ex';
import ScalingComponentDetailDrawer from './scaling-component-drawer';

// Constants
const SIZE_PER_PAGE_OPTIONS = [10, 50, 100, 200, 500];

// Utils
async function getScalingComponents(): Promise<ScalingComponentDefinitionEx[]> {
  const scalingComponents =
    await ScalingComponentService.getScalingComponents();
  return scalingComponents;
}

export default function ScalingComponentsPage() {
  const router = useRouter();
  const searchParams = useSearchParams();

  // Query params
  const pageParam = searchParams.get('page');
  const currentPage = pageParam ? parseInt(pageParam, 10) : 1;
  const sizeParam = searchParams.get('size');
  const pageSize = sizeParam
    ? parseInt(sizeParam, 10)
    : SIZE_PER_PAGE_OPTIONS[0];

  // States
  const [scalingComponents, setScalingComponents] = useState<
    ScalingComponentDefinitionEx[]
  >([]);
  const [selectedScalingComponent, setSelectedScalingComponent] =
    useState<ScalingComponentDefinitionEx>();
  const [forceFetch, setForceFetch] = useState(false);
  const [detailModalVisible, setDetailModalVisible] = useState(false);
  const [pagination, setPagination] = useState<Pagination>(new Pagination());
  const metricsPerPage = useMemo(() => {
    return scalingComponents.slice(
      (pagination.currentPage - 1) * pagination.pageSize,
      pagination.currentPage * pagination.pageSize
    );
  }, [scalingComponents, pagination.currentPage, pagination.pageSize]);
  // Effects
  useEffect(() => {
    setPagination(
      new Pagination(pageSize, scalingComponents?.length, currentPage)
    );
  }, [scalingComponents, currentPage, pageSize]);

  useEffect(() => {
    fetchMetrics();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (forceFetch) {
      fetchMetrics();
      setForceFetch(false);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [forceFetch]);

  // Handlers
  const fetchMetrics = async () => {
    try {
      const id = selectedScalingComponent?.db_id;
      let scalingComponentsData = await getScalingComponents();
      setScalingComponents(scalingComponentsData);
      setSelectedScalingComponent(
        scalingComponentsData.find(
          (item: ScalingComponentDefinitionEx) => item.db_id === id
        )
      );
    } catch (error) {
      console.error({ error });
      return [];
    }
  };

  const columnHelper = createColumnHelper<ScalingComponentDefinitionEx>();

  const columns = [
    columnHelper.accessor('id', {
      header: () => 'ID',
      cell: (cell) => cell.getValue(),
    }),
    columnHelper.accessor('component_kind', {
      header: () => 'Kind',
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
    columnHelper.accessor('updated_at', {
      header: () => 'Updated At',
      cell: (cell) => dayjs(cell.getValue()).format('YYYY-MM-DD HH:mm:ss'),
    }),
    columnHelper.display({
      id: 'actions',
      header: () => 'Actions',
      cell: (cell) => (
        <button
          className="flex h-8 items-center justify-center rounded-md border border-blue-400 bg-blue-400  pl-5 pr-5 text-sm text-gray-50"
          onClick={() => {
            setSelectedScalingComponent(cell.row.original);
            setDetailModalVisible(true);
          }}
        >
          Detail
        </button>
      ),
    }),
  ];

  return (
    <main className="flex h-full w-full flex-row">
      <div className="flex h-full w-full flex-col">
        <div className="flex h-full w-full flex-col">
          <ContentHeader
            type="OUTER"
            title="Scaling Components"
            right={
              <div className="flex items-center">
                <button
                  className="flex h-8 items-center justify-center rounded-md border border-blue-400 bg-blue-400  pl-5 pr-5 text-sm text-gray-50"
                  onClick={() => {
                    setSelectedScalingComponent(undefined);
                    setDetailModalVisible(true);
                  }}
                >
                  ADD COMPONENT
                </button>
              </div>
            }
          />
          <div className="flex w-full flex-col">
            <WATable<ScalingComponentDefinitionEx>
              tableOptions={{
                data: metricsPerPage,
                columns,
              }}
              pagination={pagination}
              onChangePage={(newPage) => {
                router.push(
                  `/app/scaling-components?page=${newPage}&size=${pageSize}`
                );
              }}
              onChangePageSize={(newSize) => {
                router.push(
                  `/app/scaling-components?page=${currentPage}&size=${newSize}`
                );
              }}
            />
          </div>
        </div>
      </div>
      {detailModalVisible ? (
        <ScalingComponentDetailDrawer
          scalingComponent={selectedScalingComponent}
          onClose={() => setDetailModalVisible(false)}
          onChange={() => setForceFetch(true)}
        />
      ) : null}
    </main>
  );
}
