# S11: Windows CI 개발 설치 산출물 준비

- 상태: in-progress
- 분류: feature
- 우선순위: P1
- 요구사항: R05, R08, R10, R11
- 담당/수정 범위: root .github/workflows/ci.yml 및 필요한 scripts·문서; timeline_precision은 /tmp 후보만 준비한다. Cargo/schema/lock 변경 없음.
- 추가 파일 담당: root scripts/ci_whole_receipt.py·scripts/test_ci_whole_receipt.py를 독립 검토·통합한다. diagnostic_inputs의 후보와 runner는 /tmp에서만 작성하며 Windows 실행 미검증 runner는 workflow에 연결하지 않는다.
- 선행 작업: 기존 Windows 우선 CI·검증된 unsigned payload 생성/ZIP 도구. 원격 실행과 공개 배포는 별도 미완료 조건이다.

## 사용자 결과

Windows CI가 실제 빌드한 개발 설치 파일을 SHA와 receipt에 묶어 검토할 수 있게 한다. 로컬 기존 바이너리를 CI 산출물로 표시하지 않는다.

## 계약과 범위

기존 .git을 보존하며 외부 저장소나 릴리스를 게시하지 않는다. 서명·legal coverage·clean OS 검증 없이 release_ready를 승격하지 않는다. 에이전트는 독립 CI 파일과 도구를 검토하고 root가 통합한다.

## 인수 조건

실제 CI 빌드 출력의 exact SHA/namespace/notice binding을 사용하는 좁은 후보를 검토한다. 로컬 도구 테스트는 실제 명령으로 확인하고 원격 Actions 성공은 별도로 남긴다.

## 검증과 handoff

현재 계약 기록. 원격 저장소 주소와 실제 CI 실행은 미확정이며 독립 개발을 계속한다.

Root는 Windows installer/companion을 명시적으로 release 빌드하고 네 실행 파일·Cargo.lock·toolchain의 SHA/bytes를 기록하는 workflow 후보를 통합했다. 동일하게 pinned upload-artifact를 사용하며 권한은 contents:read 그대로다. 실제 runner 출력만 수집한다. 현재 산출물은 build inputs이며 packaged_install_payload=false다. payload notice/graph binding과 설치 ZIP을 CI에서 생성하는 단계는 후속이다. PyYAML 구조 파싱 및 step 이름·Windows 조건을 확인했으며 원격 실행은 수행하지 않았다. 추가 의존성은 없고 컴파일/7일 artifact 저장 비용은 CI에서 측정 전이다.

Root 실제 검증: `python3 -m unittest discover -s scripts -p 'test_*install_payload.py'` 18tests PASS. 생성/패키징 실패 경로 회귀이며 새 workflow의 원격 성공이나 정식 설치 인수 증거는 아니다.

CI graph-format 보조 도구와 실패 경로 회귀12개를 root가 통합·실제 실행했다. 기존 실제 Windows graph payload로 실행하면서 Cargo의 proc-macro annotation 및 이름과 동일한 디렉터리의 short path package ID를 거절하는 오류를 발견했다. 각 실패를 재현하는 회귀가 먼저 실패했고 정확한 두 Cargo 형식만 허용한 뒤12tests PASS. 현재 captured graph258과 metadata/Cargo.lock SHA를 별도로 대조했다. [형식 결과](../validation/evidence/windows-ci-format-existing-local-graph-20261011.json)는 기존 로컬 출력의 형식 검증이며 remote_execution_verified=false·legal_coverage_complete=false를 유지한다. 새 CI가 실제 stdout/stderr를 분리 수집해 같은 형식 도구로 기록하도록 구성했다. 현재 notice 원본 checksum·font binding·CI whole receipt constructor가 미완료여서 build-input artifact를 설치 ZIP으로 표시하지 않는다. Python 표준 모듈과 기존 TOML fallback만 사용하고 제품 런타임 의존성은 없다.

원 crate checksum을 trust root로 삼는 registry 원본 validator를 통합했다. `.cargo-checksum.json`의 package 문자열만으로 mutable files map을 믿지 않으며, 원 `.crate` 전체 SHA가 Cargo.lock과 일치하고 tar의 요구 파일 byte와 추출 원본을 모두 비교한다. symlink/reparse attribute·archive 원 HANDLE identity·256MiB/276MiB 상한·malformed 입력 회귀13개 PASS. 기존 로컬 epaint_default_fonts0.33.3 원 archive와 font4개/notice4개/Cargo.toml/src-lib의10개 요구 파일 byte/size/SHA를 실제 실행으로 검증했다. [범위 증거](../validation/evidence/ci-format-local-font-archive-binding-20261011.json). 이는 embedded 선언 전체·linked payload·전체 dependency notice·실제 Windows reparse·원격 CI 성공을 입증하지 않으며 모든 false readiness를 유지한다. CI docs job에13개 실패 회귀를 연결했다.

새 Git source bundle090595d에는 현재 build-input/graph/registry 원 byte validator·회귀·workflow 변경을 포함했다. 원격 CI 성공과 notice/font/VCS 전체 collector, CI whole producer·검증 constructor가 남으며 별도의 format receipt 후보를 실제 runner provenance로 승격하지 않는다. 현재 프로젝트 docs886 및 exportdocs830 PASS를 확인했다.

2026-10-11 후속: root는 bounded CI receipt 형식 생성기를 통합했다. Windows ADS/device/trailing-dot·space/reparse, 중복 로그·hardlink binary, 원 열린 파일 변경을 거절한다. 실제 명령 argv/exit와14개 원 로그·4개 binary·추적 파일의 before/after SHA를 묶지만 caller가 공급한 identity를 CI 인증으로 취급하지 않는다. `python3 scripts/test_ci_whole_receipt.py`12개 독립 회귀 PASS(하위 후보의 상속 중복7개는 제거). reparse attribute·동시 file 변경은 모의 회귀이며 실제 Windows 원 Job/runner 증거가 아니다. workflow에는 이 형식 회귀만 연결했다. Windows raw argv runner는 native 미검증이라 /tmp에 유지하며, 실제 CI 수집·설치 패키지 생성은 여전히 미완료다. remote_execution_authenticated/private_job_cleanup_verified/native_gui_acceptance_verified/legal_coverage_complete/release_ready 모두 false다. 추가 제품 의존성이나 런타임 비용은 없으며 Python의 입력 상한은 추적 파일256MiB·개별16MiB/로그8MiB/binary128MiB다. 새 두 helper는090595d 이후 변경으로 기존 Git bundle에 포함됐다고 보고하지 않는다.

후속 raw producer는 프로젝트에 통합했고 Windows에서 실제Cargo fmt의 gated Job 선배정·natural0·원 raw drain을 확인했다. CI 관련 회귀44개와 Windows7개 회귀 PASS다. 핵심7개 기존 Windows 작업을 같은 producer로 대체하여 중복 빌드를 추가하지 않았고 실패raw/lifecycle는 always artifact로 보존한다. 새 68c4849 Git 묶음은 이 변경을 포함한다. 모든7개 actualproducer 작업·remote Actions·font/notice 전체binding·CI 설치 ZIP 생성은 남아 있다.
