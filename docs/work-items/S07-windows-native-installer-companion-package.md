# S07: 현재 Windows 개발 ZIP에 설치 화면 포함

- 상태: done (세 binary 개발 ZIP scoped 완료; 법적 완비·정식 installer·전체 P0 제외)
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — 프로젝트 통합·검증·문서; diagnostic_cache — /tmp packager 초안과 원 파일 pin만; metrics_recovery — /tmp 기존 test 파일의 실패 회귀 초안만. 같은 프로젝트 파일 동시 수정 없음.
- 선행 작업: 현재 Windows whole proof 및 기존 MSVC CPU/CLI companion packager. 현재 재시작 검사 인수와 성능 검증 중에는 제품 소스·현재 공급물 pin을 변경하지 않는다.

## 사용자 결과

개발용 ZIP에서 앱·개발 CLI와 같은 proof의 네이티브 설치 화면을 함께 받을 수 있다.

## 계약과 범위

후보 파일은 scripts/windows_msvc_candidate_package.py, scripts/test_windows_msvc_candidate_package.py, 관련 개발 안내 및 이 문서다. 기존 두 binary 호출을 유지하며 installer를 선택적으로 포함한다. exact whole artifact3 SHA/size·AMD64 GUI subsystem 역할 검증, immutable copy와 bounded/create-only 출력을 유지한다. 공유 schema/Cargo/lock 변경은 root가 조정한다. 정식 서명·법적 완비·clean OS·공개 출시 완료로 표현하지 않는다.

## 인수 조건

현재 whole에 결합된 fresh ZIP 두 개의 전체 member/hash/CRC/mode/timestamp 결정성과 모든 세 binary의 PE 역할을 검사한다. source/proof 불일치·missing/wrongSHA/subsystem installer·entry collision·파일 증가 실패를 검증한다. 전체 lock notice 누락과 release_ready=false를 보존한다. 과거 두 binary ZIP을 새 공급물 검증으로 소급하지 않는다.

## 검증과 handoff

독립 readonly 감사에서 현재 notice inventory records526/missing43/completefalse를 확인했다. /tmp 초안을 준비 중이며 프로젝트 구현·새 ZIP 생성은 아직 하지 않았다. 새 의존성이나 전역 도구 설치는 제안하지 않았다.

초안 파일 담당을 분리했다. diagnostic_cache는 packager만, metrics_recovery는 test만 준비한다. packager 초안은 AST 검사만 완료했으며 실제 통합·패키징·현재 graph 검증은 아직 하지 않았다.

### 통합 및 실행한 검사

원본/초안 SHA를 확인한 뒤 선택적 `--native-installer-exe`·`--native-installer-sha256` 쌍과 역할별 PE 검증·proof3 결합을 통합했다. root는 일반 project import로 시험을 게시하고 실제 파일 증가를 탐지하며 실패 ZIP·원 파일을 보존하는 회귀를 추가했다. `PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_windows_msvc_candidate_package.py` 실제 11 tests/OK (0.080s). 이 시험의 native proof/notice/PE 함수는 mock이며 실제 PE 인수로 확대하지 않는다. 새 Windows graph는 실제 tree2·metadata1 명령 exit0로 수집됐으며 원 map/lock/artifact를 보존했다. 실제 개발 ZIP 두 개 생성과 root/독립 전체 member 인수는 아직 진행 중이다. Cargo·schema·의존성 추가 없음.

### 실제 생성 및 root 인수

두 실제 개발 ZIP SHA `45c4d500788a82304c9f34cfea41242dbce1a4f8b2acdfe18ddd8b7331d7e3bf`, 13,043,842B·466 entries·nonbinary 2,487,546B가 일치했다. root는 모든 member SHA/CRC/mode/고정 timestamp와 세 실제 실행 파일의 AMD64 PE32+/GUI2·console3 역할, 현재 whole/graph/Rust592 map 및 독립 notice inventory records258/selected missing0·전체lock missing43을 대조했다. 근거: `../validation/evidence/windows-msvc-native-installer-companion-graph-20261006-continue/package-root-review.json`. 원 source576 두 binary ZIP보다990,439B 커졌지만 여러 binary 및 metadata 변경을 포함하므로 installer 기능만의 증가로 해석하지 않는다. installer 자체는799,744B이며 ZIP 포함만으로 실행되는 프로세스·thread·런타임 비용은 추가하지 않는다. 새 dependencies/schema/Cargo.lock 변경 없음. 아직 독립 ZIP 검토는 진행 중이다. 법적 완비·실제 linked closure·static CRT/std 공지·서명·clean OS·정식 installer와 release_ready는 false다.

### 최종 독립 인수

root 전체 member 결과와 독립 검토의 현재 source592 전체 map·원 artifacts3·whole4/graph3 raw lifecycle·두 ZIP466 entries·각 member 전체 SHA/size가 일치했다. 독립 검토는 selected258 records 및455 notice 원 cached/upstream/source bytes를 대조했다. 근거: `../validation/evidence/windows-msvc-native-installer-companion-graph-20261006-continue/package-root-completion-review.json` 및 `package-independent-review.json`. 이 작업은 완료됐으며 정식installer/defaults/browse/upgrade, CRT/std 공지·서명·clean OS와 전체P0는 후속이다. root archive 인수 첫 시도는 entries를 count로 오해해 쓰기 전에 AssertionError로 중단했고 dictionary 전체를 root SHA/size map과 대조해 통과시켰다. 실패를 성공으로 변환하지 않았다.
