# R10/R11: 소유 incremental 후보 native reparse 읽기 검증

- 상태: in-progress — Root 읽기 전용 실행 예정; 삭제 미수행
- 분류: development-environment validation
- 우선순위: P0 검증 선행
- 담당/수정 범위: search_reveal; owned-incremental-native-reparse-v1 scratch만
- 선행: Root가 모든 Cargo terminal과 target 사용중단을 확인한 뒤 실행; 원 read-only inventory 고정후보12개

## 사용자 결과

소유 Windows incremental cache 후보가 native reparse/special 없이 원 manifest와 같은 오래된 디렉터리인지 확인한다. 삭제/회수 작업을 수행하거나 승인하지 않는다.

## 계약과 범위

고정 target-msvc marker와 exact incremental root의 고정12직접자식 이름만 받는다. 이름/prefix/순서를 원 manifest SHA와 함께 하드 바인딩한다. D:\부터 incremental ancestor와 하위디렉터리는 OPEN_REPARSE_POINT/BACKUP_SEMANTICS/read attributes 핸들로 보존한다. share READ only로 rename/write 공유충돌은 거절하며 native GetFileType disk/AttributeTag/FILE_ID_INFO/Basic/Standard 정보를 확인한다. reparse/device/non-disk/delete-pending/protected PE 확장자 즉시 거절한다. 모든 디렉터리 nativeID·type·size·mtime·attributes 전후 동일과 final path를 확인한다. marker162B/SHA/ID·manifestSHA/ID도 전후 확인한다. 원 cache/manifest/Live/S/docs/E/Cargo 변경0.

## 인수 조건

후보별 실제 file/dir/entry/hardlink count·logical bytes·root/newest native write time·root nativeID를 기록하고 원 manifest count/bytes/mtime가 다르면 stale FAIL이다.24시간 조건도 실행시 실제 UTC로 확인한다. 전체20000entry/256dir/90초/하위깊이8/후보5000entry/출력60000문자 상한, compile45초/native host120초를 유지한다. 산출물은 bounded receipt와 Root review 대기 상태이며 삭제 권한/실제 물리 회수량/원자적 snapshot을 주장하지 않는다. reparse가 없더라도 Root의 별도 소유권·stale 재검토를 대체하지 않는다.

## 검증과 handoff

argv-template.json의 fresh output을 Root가 지정한다. 기존 Windows Job host/csc를 사용하며 probe 자체는 GUI/desktop/입력/네트워크 API0, cache쓰기/삭제 API0이다. 준비 시 Python AST1만 확인; C# compile/native/Cargo0. Linux lstat 통과를 native 통과로 쓰지 않는다. 실패는 원 stage/code만 기록하며 외부경로 확장/visible desktop/global input fallback0이다.

Root native 재실행v2는 compile/native0,6270entry/46directorypins/12후보가 원manifest와 일치했다. 최초 sandbox 실행은 WSL vsock 시작 실패로 보존했다. Root가 전Cargo 종료·소유marker·metadata digest·24h·비reparse·default3/675source를 재확인하고 고정12개 오래된incremental만 정리했다. 첫항목 후 create-only 결과기록 함수의 재쓰기 오류로 중단해 원partial을 보존하고, 별도v3에서 남은11개를 다시 검증해 정리했다. v3 actual exit0/11삭제, 여유2,356,867,072→6,090,633,216B(차이3,733,766,144B). 첫항목을 포함한 총12개 부재와 원source/default3/marker 보존을 확인했다. 논리5.33GB와 실제 회수는 다르며, 동시 외부디스크사용은 측정하지 않았다. 사용자데이터/증거/PE는 대상아니다. 결과 `.local-tools/session-recovery-20261013/owned-incremental-cleanup-20261016-v3/result.json`.
