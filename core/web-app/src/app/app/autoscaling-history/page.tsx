'use client';

import React, { useEffect, useMemo, useState } from 'react';
import { useSearchParams, useRouter, usePathname } from 'next/navigation';
import dayjs, { Dayjs } from 'dayjs';
import { decodeTime } from 'ulid';
import AutoscalingHistoryService from '@/services/autoscaling-history';
import { AutoscalingHistoryDefinition } from '@/types/bindings/autoscaling-history-definition';
import AutoscalingHistoryHeatmap from './autoscaling-history-heatmap';
import AutoscalingHistoryDetailDrawer from './autoscaling-history-drawer';
import ContentHeader from '../common/content-header';
import { TableComponent } from '../common/table';
import { AutoscalingHistoryDefinitionEx } from './autoscaling-history-definition-ex';

// Default Values
const DEFAULT_FROM = dayjs().subtract(7, 'days');
const DEFAULT_TO = dayjs();
const SIZE_PER_PAGE_OPTIONS = [10, 50, 100, 200, 500];

// Utils
const formatDate = (date: Dayjs) => date.format('YYYY-MM-DD');

async function getAutoscalingHistory(from: Dayjs, to: Dayjs) {
  const autoscalingHistory =
    await AutoscalingHistoryService.getAutoscalingHistoryByFromTo(from, to);
  return autoscalingHistory;
}

export default function AutoscalingHistoryPage() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  // Query Params
  const fromParam = searchParams.get('from');
  const toParam = searchParams.get('to');
  const from = fromParam || formatDate(DEFAULT_FROM);
  const to = toParam || formatDate(DEFAULT_TO);
  const fromDayjs = useMemo(() => dayjs(from).startOf('day'), [from]);
  const toDayjs = useMemo(() => dayjs(to).endOf('day'), [to]);
  const pageParam = searchParams.get('page');
  const page = pageParam ? parseInt(pageParam, 10) : 1;
  const sizeParam = searchParams.get('size');
  const size = sizeParam ? parseInt(sizeParam, 10) : 10;

  // UI State
  const [selectAll, setSelectAll] = useState(false);
  const [sizePerPage, setSizePerPage] = useState(SIZE_PER_PAGE_OPTIONS[0]);
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPage, setTotalPage] = useState(1);
  const [detailModalVisible, setDetailModalVisible] = useState(false);
  const [fetchFlag, setFetchFlag] = useState(false);

  // Data
  const [autoscalingHistory, setAutoscalingHistory] = useState<
    AutoscalingHistoryDefinitionEx[]
  >([]);
  const [selectedAutoscalingHistory, setSelectedAutoscalingHistory] =
    useState<AutoscalingHistoryDefinitionEx>();

  //
  // Effects
  //

  // Fetch Autoscaling History Data when the page is loaded with from and to params
  useEffect(() => {
    fetchAutoscalingHistory();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [fromDayjs, toDayjs]);

  // Update total page when autoscaling history data is updated
  useEffect(() => {
    setTotalPage(Math.ceil(autoscalingHistory.length / sizePerPage) || 1);
  }, [autoscalingHistory, from, to, currentPage, totalPage, sizePerPage]);

  // Update current page and size per page when page and size params are updated
  useEffect(() => {
    setCurrentPage(page || 1);
    setSizePerPage(size || SIZE_PER_PAGE_OPTIONS[0]);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [page, size]);

  // Update URL when current page is updated
  useEffect(() => {
    if (currentPage > totalPage || currentPage < 1) {
      setCurrentPage(1);
    }
    router.push(
      `${pathname}?from=${from}&to=${to}&page=${currentPage}&size=${sizePerPage}`
    );
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    autoscalingHistory,
    from,
    to,
    currentPage,
    totalPage,
    sizePerPage,
    searchParams,
  ]);

  // ?????
  useEffect(() => {
    if (fetchFlag) {
      fetchAutoscalingHistory();
      setFetchFlag(false);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [fetchFlag]);

  // Fetch Autoscaling History Data
  const fetchAutoscalingHistory = async () => {
    try {
      const id = selectedAutoscalingHistory?.id;
      let autoscalingHistoryData = await getAutoscalingHistory(
        fromDayjs,
        toDayjs
      );
      autoscalingHistoryData = autoscalingHistoryData.map(
        (autoscalingHistoryDataItem: AutoscalingHistoryDefinition) =>
          ({
            ...autoscalingHistoryDataItem,
            created_at: decodeTime(autoscalingHistoryDataItem.id),
            is_checked: false,
          } as AutoscalingHistoryDefinitionEx)
      );
      setAutoscalingHistory(autoscalingHistoryData);
      setSelectedAutoscalingHistory(
        autoscalingHistoryData.find(
          (item: AutoscalingHistoryDefinition) => item.id === id
        )
      );
    } catch (error) {
      console.error({ error });
    }
  };

  // Date Picker Handler
  const handleDate = (field: 'from' | 'to', value: string) => {
    const newDate = dayjs(value);
    const isValidDate = newDate.isValid();

    if (!isValidDate) {
      return;
    }
    const params = {
      from: field === 'from' ? formatDate(newDate) : from,
      to: field === 'to' ? formatDate(newDate) : to,
    };
    router.push(
      `${pathname}?from=${params.from}&to=${params.to}&page=1&view=${sizePerPage}`
    );
  };

  // Select All Handler
  const handleSelectAll = (event: React.ChangeEvent<HTMLInputElement>) => {
    const checked = event.target.checked;
    setSelectAll(checked);
    const updatedAutoscalingHistoryData = autoscalingHistory.map(
      (updatedAutoscalingHistoryDataItem) => ({
        ...updatedAutoscalingHistoryDataItem,
        is_checked: checked,
      })
    );
    setAutoscalingHistory(updatedAutoscalingHistoryData);
  };

  const handleSizePerPage = (event: React.ChangeEvent<HTMLSelectElement>) => {
    const newItemsPerPage = parseInt(event.target.value, 10);
    setSizePerPage(newItemsPerPage);
    setCurrentPage(1);
  };

  const handleCurrentPage = (newPage: number) => {
    setCurrentPage(newPage);
  };

  const onClickDetails = (
    autoscalingHistoryItem: AutoscalingHistoryDefinitionEx | undefined
  ) => {
    setSelectedAutoscalingHistory(autoscalingHistoryItem);
    setDetailModalVisible(true);
  };

  const tableFormat = [
    {
      label: 'checkbox',
      type: 'checkbox',
      content: 'is_checked',
      weight: '1',
      status: selectAll,
      function: handleSelectAll,
    },
    {
      label: 'Scaling Plan ID',
      type: 'span',
      content: 'plan_id',
      format: 'plain',
      weight: '4',
    },
    {
      label: 'Metric Values',
      type: 'span',
      content: 'metric_values_json',
      format: 'json',
      weight: '10',
    },
    {
      label: 'Metadata Values',
      type: 'span',
      content: 'metadata_values_json',
      format: 'json',
      weight: '5',
    },
    {
      label: 'Fail Message',
      type: 'span',
      content: 'fail_message',
      weight: '3',
    },
    {
      label: 'Status',
      type: 'button',
      content: 'fail_message',
      format: 'alternate',
      weight: '1',
      function: onClickDetails,
    },
    {
      label: 'Date',
      type: 'span',
      content: 'created_at',
      format: 'unix',
      weight: '2',
    },
    {
      label: 'Action',
      type: 'button',
      content: 'dataItem',
      format: 'click',
      weight: '1',
      function: onClickDetails,
    },
  ];

  return (
    <main className="flex h-full w-full flex-row">
      <div className="flex h-full w-full flex-col">
        <div className="flex h-full w-full flex-col">
          <ContentHeader
            type="OUTER"
            title="Autoscaling History"
            right={
              <div className="flex items-center">
                <div className="form-control mr-2">
                  <label className="input-group-sm">
                    <input
                      type="date"
                      className="input-bordered input input-sm max-w-[130px] cursor-text px-2 text-center focus:outline-none"
                      max={formatDate(toDayjs)}
                      value={from}
                      onChange={(event) =>
                        handleDate('from', event.target.value)
                      }
                    />
                  </label>
                </div>
                <span>-</span>
                <div className="form-control ml-2">
                  <label className="input-group-sm">
                    <input
                      type="date"
                      className="input-bordered input input-sm max-w-[130px] cursor-text px-2 text-center focus:outline-none"
                      min={formatDate(fromDayjs)}
                      value={to}
                      onChange={(event) => handleDate('to', event.target.value)}
                    />
                  </label>
                </div>
              </div>
            }
          />
          <div className="flex w-full flex-col">
            <AutoscalingHistoryHeatmap
              autoscalingHistory={autoscalingHistory}
              from={fromDayjs}
              to={toDayjs}
            />
            <TableComponent
              tableFormat={tableFormat}
              /*  */
              data={autoscalingHistory}
              /*  */
              sizePerPageOptions={SIZE_PER_PAGE_OPTIONS}
              sizePerPage={sizePerPage}
              handleSizePerPage={handleSizePerPage}
              /*  */
              currentPage={currentPage}
              totalPage={totalPage}
              handleCurrentPage={handleCurrentPage}
            />
          </div>
        </div>
        {detailModalVisible ? (
          <AutoscalingHistoryDetailDrawer
            autoscalingHistoryItem={selectedAutoscalingHistory}
            onVisibility={setDetailModalVisible}
          />
        ) : null}
      </div>
    </main>
  );
}
