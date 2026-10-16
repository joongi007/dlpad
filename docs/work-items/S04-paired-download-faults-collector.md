# S04: Windows paired 다운로드 실패 수집기

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R08, R10, R11
- 담당/수정 범위: root; scripts/windows_paired_download_faults_probe.py, CONTEXT.md 및 본 문서
- 선행 작업: paired public fixture 실패/취소 확장; 프로젝트 전용 Windows CLI host

## 사용자 결과

인증과 관리 도구를 함께 쓰는 다운로드 실패·취소가 실제 Windows에서 소유 프로세스 정리와 원본 보존을 유지하는지 검증한다.

## 계약과 범위

공개 Repository 합성 fixture만 사용한다. 기존 private desktop CLI host를 재사용하며 사용자 데스크톱 fallback과 실제 네트워크·계정·미디어를 사용하지 않는다. Cargo/source hash 고정 상태에서 빌드 후 metadata 실패/download 실패/취소를 순차 실행한다. 자동 startup 복구와 부모 crash는 별도다.

## 인수 조건

각 native 실행 exit0, fixture의 해당 scenario 및 passed=true, 합성 입력·인증 출력 생략·DB privacy·원본 보존·소유 기록 정리와 재사용이 확인되어야 한다. 부분 실행을 전체 성공으로 보고하지 않는다. source 및 binary SHA 불일치, 출력/시간 상한 초과는 실패다.

## 검증과 handoff

구현 중. native 실행은 source freeze와 fixture handoff 후 주 담당이 수행한다.

주 담당 실제 Windows 인수: `windows-paired-public-download-faults-first`에서 metadata exit41/Failed, download exit42/Failed, actual reader 취소/Cancelled 세 경로가 통과했다. `root-review.json`에서 원 DB download_jobs 상태와 artifact0, 원 cookie/tool SHA, sentinel, receipt 사본 SHA/256바이트/managed magic, cookie cache 소유 기록 부재와 managed 9파일/pending 부재를 독립 확인했다. 첫 root SQL의 jobs 테이블명 오류는 실제 download_jobs schema로 정정했다. 최초 workspace strict Clippy는 fixture의 collapsible_if 경고로 실패하여 조건문을 동등한 let-chain으로 수정했다. 원 실패 증거를 보존하고 최신 전체 검증은 별도 재실행 중이다. 부모 crash/자동 startup 복구는 아직 미완료다.

최종 재검증: `windows-paired-faults-workspace-clippy-retry` Windows GNU CPU fmt/clippy/test/release 모두 PASS, 각 final source SHA를 현재 소스와 대조했다. 수정 후 `windows-paired-public-download-faults-clippy-retry` 실제 세 native case도 재통과하고 root-review.json의 원 DB/receipt SHA/원본·sentinel/소유 기록 부재 대조가 통과했다. 문서 검사 604개 PASS. 부모 crash/자동 회수는 후속이며 전체 P0 완료가 아니다.
