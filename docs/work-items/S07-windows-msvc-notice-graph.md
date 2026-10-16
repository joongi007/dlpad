# S07: MSVC 앱·CLI의 대상별 notice graph

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R11, R08
- 담당/수정 범위: root — collector 통합·native 실행·공유 Cargo/lock·증거/문서; metrics_recovery — /tmp collector/parser 초안; 독립 검토는 실제 수집 후 별도 배정
- 선행 작업: S07-windows-msvc-workspace-experiment의 현재 source576 전체4검사·MSVC CPU/CLI 핀.

## 사용자 결과

MSVC 배포 후보에 실제 대상 graph의 dependency notice를 포함할 수 있도록 GNU 결과와 구분한 근거를 만든다.

## 계약과 범위

새 scripts/windows_msvc_notice_graph_probe.py에서 원 private Windows host로 Cargo tree app(cpu)/engine(normal,build) 및 metadata(filter MSVC)를 직렬 수집한다. 원 effective configuration과 locked graph를 유지하고 tree 각1MiB/metadata8MiB 상한, source/원GNU·MSVC CPU/CLI/lock 보존을 확인한다. 초안은 /tmp, 프로젝트 수정과 실행은 root 담당이며 다른 담당과 파일 동시 수정은 없다. 기존 GNU package 계약은 수정하지 않는다. resolved package graph는 linker가 실제 남긴 코드 목록이 아니며 full lock의 미사용 notice 누락과 구분한다. schema/dependency/lock 변경은 없다.

## 인수 조건

세 실제 명령 exit0·자연 Job0·private/input 보존, 현재 source/CPU/CLI 핀, exact package key와 lock/metadata 출처 매핑, 원 raw 출력과 bounded parser 보존, root/독립 검토. 누락된 대상 notice를 삭제해 성공으로 만들지 않는다. MSVC package·static CRT/std 재배포 고지·법률/서명·cleanOS는 후속 별도 인수다.

## 검증과 handoff

collector 초안 작성 중이다. 이전 GNU graph/ZIP 완료를 MSVC 대상으로 소급하지 않는다.

실제 [MSVC graph](../validation/evidence/windows-msvc-notice-graph-20261006-continue/result.json)의 app-tree/engine-tree/metadata 세 명령을 원 Job/private/input 보호와 함께 통과했다. app258·engine86·union258 exact keys와 metadata source/ID·Cargo.lock checksum 대응을 root/독립 담당이 대조했다. root는 원 raw/source576·전체 input/CPU/CLI/GNU/lock 핀을 직접 확인하고 기존 bounded notice helper로 selected258 records의 모든 notice SHA/내용 존재를 다시 확인했다. selected 누락0, 전체lock 별도 누락43은 유지한다. source graph가 실제 linker의 dead-code 제거·embedded 저작권·static CRT/std 전체고지·법적/서명/공개 릴리스 완비를 증명하지 않으며 별도 root/independent review를 저장했다.
