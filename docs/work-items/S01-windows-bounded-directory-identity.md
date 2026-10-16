# S01: Windows bounded directory identity 실험

- 상태: in-progress
- 분류: performance
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root 새 crates/engine/examples/windows_directory_identity_cost.rs·전용 증거·문서, diagnostic_cache /tmp Rust draft만, metrics_recovery 독립 안전성 검토 /tmp만
- 선행 작업: S01-windows-library-identity-cost 실측과 DirEntry.metadata 캐시 대체 안전성 거절

## 사용자 결과

원 native directory HANDLE에서 bounded batch로 이름·크기·시간·opaque128bitID를 얻는 API를 실제 시험한다. 제품 scanner의 대체 채택은 별도 안전성 인수 뒤 결정한다.

## 계약과 범위

GetFileInformationByHandleEx FileIdExtdDirectoryRestartInfo/Info, 고정64KiB buffer와 bounded record parser, retained own directory File/volumeID/finalpath 검증. 이름 길이·UTF16·offset·alignment·수/크기·reparse 제한을 검사한다. owned synthetic10000파일만 접근하고 cancellation/deadline·최대batch/entry와 source/binarypin을 지킨다. Cargo/schema/deps와 생산scanner 변경0. 버퍼 parser의 실패 회귀를 실제 실행하고 개별 파일 HANDLE의 원ID/size/mtime와 대조한다. 사용자의 기존 데이터·실제영상은 쓰지 않는다.

## 인수 조건

Windows native compile/run; streamed batch10000관측·原HANDLE 128개 이상 실제 crosscheck·원directory identity/finalpath와 sentinel 유지; 3round CPU/wall/버퍼bytes/호출수를 원raw로 기록. path containment·snapshot races·unsupported filesystem은 한계로 기록하고 성능 성공만으로 생산채택하지 않는다. 실제 생산reconcile/GUI budget은 별도 인수다.

## 검증과 handoff

DirEntry.metadata 단순 교체는 삭제/교체 후 stale metadata와 디렉터리 reparse 위험 때문에 채택하지 않는다. 참고: Microsoft GetFileInformationByHandleEx와 FILE_ID_EXTD_DIR_INFO 문서. 이 작업은 원OS 관측 capability와 parser/비용 검증 범위다.

2026-10-05 원343줄 초안은 64KiB buffer·integer field read_unaligned·offset/name bounds·streamed batches·128개원fileID/size/FILETIME 교차확인·meaningful malformed parser 회귀를 포함한다. root가 전체초안읽기검토했고 unused File import는 게시시제거한다. 현재 원nativeGUI sourcefreeze 종료까지 Rust게시대기이며 실제성능/생산안전동등성은 미검증이다.

첫 실제Windows parser회귀/빌드/3회cost PASS: 10000/21batches/128crosscheck 각회, API+parsewall2.375~2.866ms/원wholeCPU15.625ms/전체wall13.786~14.667ms. root가 원counter/원source와 naturalJob0/private/input/sentinel을 검토했다. 이 값은 128개교차확인만 포함하며 생산containment·race동등성을 뜻하지 않는다. 고정fixture명 bitset과 각batch 원directory finalpath 재검증을 다음보강으로 진행한다. 성능값은단일prototype이며 actualGUI예산개선은 미검증이다.
