'use client';

import React, { useEffect, useMemo, useState } from 'react';
import AutoscalingHistoryService from '@/services/autoscaling-history';
import { AutoscalingHistoryDefinition } from '@/types/bindings/autoscaling-history-definition';
import dayjs, { Dayjs } from 'dayjs';
import { useSearchParams } from 'next/navigation';

import AutoscalingHistoryDetailDrawer from '../../autoscaling-history-drawer';
import AutoscalingHistoryPage from '../page';

const DATE_FORMAT = 'YYYY-MM-DD';
const DEFAULT_FROM = dayjs().subtract(7, 'days');
const DEFAULT_TO = dayjs();

const formatDate = (date: Dayjs) => date.format(DATE_FORMAT);

async function getAutoscalingHistoryDefinition(from: Dayjs, to: Dayjs) {
  try {
    const autoscalingHistoryDefinition =
      await AutoscalingHistoryService.getHistoryByFromTo(from, to);
    return autoscalingHistoryDefinition;
  } catch (error) {
    console.error(error);
  }
}

interface AutoscalingHistoryDefinitionEx extends AutoscalingHistoryDefinition {
  created_at: string;
}

export default function AutoscalingHistoryDetailLayout({
  children,
  params: { id: id },
}: {
  children: React.ReactNode;
  params: { id: string };
}) {
  const searchParams = useSearchParams();
  const fromParam = searchParams.get('from');
  const toParam = searchParams.get('to');
  const from = fromParam || formatDate(DEFAULT_FROM);
  const to = toParam || formatDate(DEFAULT_TO);
  const fromDayjs = useMemo(() => dayjs(from), [from]);
  const toDayjs = useMemo(() => dayjs(to).endOf('day'), [to]);

  const [history, setHistory] = useState<AutoscalingHistoryDefinitionEx>();

  const findAutoscalingHistoryDefinitionById = (
    id: string,
    array: AutoscalingHistoryDefinitionEx[]
  ) => {
    for (const autoscalingHistoryDefinition of array) {
      if (autoscalingHistoryDefinition.id === id) {
        return autoscalingHistoryDefinition;
      }
    }
  };

  useEffect(() => {
    const setHistoryData = async (
      histories: AutoscalingHistoryDefinitionEx[]
    ) => {
      if (histories) {
        const foundHistory = findAutoscalingHistoryDefinitionById(
          id,
          histories
        );
        setHistory(foundHistory);
      }
    };

    const fetchHistoryData = async () => {
      try {
        const histories = await getAutoscalingHistoryDefinition(
          fromDayjs,
          toDayjs
        );
        setHistoryData(histories);
      } catch (error) {
        console.error({ error });
      }
    };

    fetchHistoryData();
  }, []);

  return (
    <div className="relative flex h-full w-full">
      <AutoscalingHistoryPage />
      <AutoscalingHistoryDetailDrawer autoscalingHistoryDefinition={history} />;
    </div>
  );
}
