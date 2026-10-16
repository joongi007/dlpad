# S03: 실제 SQLite 용량 상한 실패와 journal 복구

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R06, R13
- 담당/수정 범위: diagnostic_cache /tmp test·감사 초안; root crates/engine/src/download.rs cfg(test) 회귀·본 문서·원 Windows 실행·전체검증.
- 선행 작업: 현재 notice 원native 작업 terminal; source 동결 종료 후 test-only 게시.

## 사용자 결과

작업 추가 중 실제 SQLITE_FULL이 발생해도 기존 작업·attempt 기록과 원 합성 파일이 유지되고, DB 재열기 후 기존 active journal만 Interrupted로 복구한다.

## 계약과 범위

원 제품 retained SQLite connection에 PRAGMA max_page_count=현재page_count+8을 설정하고 실제 반환값을 확인한다. 최대512 public create_job 호출 안에서 실제 sqlite ErrorCode::DiskFull을 필수로 관측한다. 외부 connection quota를 제품 connection의 설정으로 오인하지 않는다. cfg(test) 이외 제품API/schema/dependency/런타임 비용 변화0. 임시폴더와 합성fixture만 사용하며 실제 filesystem ENOSPC/영상 다운로드/publication final save 인수가 아니다.

## 인수 조건

실패id partial row 없음, 기존Job JSON·attempt row·원합성파일/sentinel 보존, readonly-style 검토 후 repository reopen integrity_check ok. 원Preparing→Interrupted 회수1 및 다음recover0, 새Queued작업 자동실행0. 실제 Windows narrow 및 최신 전체 검증과 sourceSHA를 root 확인한다.

## 검증과 handoff

구현·실제 검증 진행 중. 실제 filesystem disk-full 및 publication 이후 finalsave FULL은 별도 인수로 남는다.

실제 Windows exact1 PASS/0FAIL, 0.11s. 최초 u64 FromSql 비지원 compile101을 보존하고 SQLite 지원 i64로 수정했다. root가 원test/nativeJob natural0/privateclose/inputSame/현재535source SHA일치를 직접 확인했다.[원 결과](../validation/evidence/windows-sqlite-full-types-fixed-20261005-resume/sqlite-full.json)、[root 검증](../validation/evidence/windows-sqlite-full-types-fixed-20261005-resume/root-review.json)。현재 전체workspace 검증 중이며 완료 판정은 terminal 뒤 수행한다.

현재 Windows 기본 CPU 전체fmt/clippy/test/release4 PASS, root source535 SHA 불일치0. [원전체](../validation/evidence/windows-notice-sqlite-current-default-20261005-resume/result.json). 이 완료는 publiccreate_job 실제FULL과기존 Preparing회수의 좁은 인수이며 publication 후 완료저장 FULL은 별도 작업이다.
