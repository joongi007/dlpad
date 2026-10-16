# S04: FFmpeg verified selection follow-up audit

- 상태: done
- 분류: audit
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당/수정 범위: diagnostic_cache; 이 문서만
- 선행 작업: root verified selection 구현; source freeze

## 사용자 결과

현재 선택은 두 version process/원본 경로 검증을 수행하지만 전체10초·취소 계약의 해시/DB 단계에 남은 gap이 있다. 이 감사 완료는 해당 gap 구현 완료가 아니다.

## 계약과 범위

Rust/Cargo/테스트 소스 수정과 실행 없음. ffmpeg_version.rs, package_inventory.rs, direct_download.rs, process.rs, 실제 합성 회귀를 읽었다.

## 관측과 구체 후속

1. **해시 budget/cancel gap**: ffmpeg_version.rs:89/132의 verify_package → package_inventory.rs:202–225는64KiB stream SHA지만 취소/시간 체크가 없다.384MiB선택 합계의 전후2pass는deadline 이후에도 읽기를 계속한다. 종료 후 checkpoint가 실패를 반환해도 응답시간 계약은 충족하지 않는다. reader에deadline/cancel check를 전달하는 verified API 후 large owned fixture 중cancel 회귀가 필요하다. blocking filesystem read 자체를 atomically interrupt한다는 주장은 불가하다.
2. **DB commit 시점 gap**: ffmpeg_version.rs:133 체크 후135–137 transaction/write/commit에는 체크가 없다. owned SQLite writer lock으로 기다리는 중 cancel/deadline이 바뀌어도 이후active가 commit될 수 있다. fixture lock→selection worker→cancel→lockrelease→active/priorbytes 대조를 제안한다. transaction acquisition/write 후commit 직전 취소 검사를 root가 조정해야 한다.
3. **stdout bound 구분**: process.rs:107/216 metadata capture 상한은8MiB다. ProcessSpec max_log65536과 다르다. ffmpeg_version.rs:113–123은OutcomeOutputLimit/truncated/exit!=0를 거절하며 순수parser는stdout64KiB/lines64/line4096/ASCII/exactfirsttool-version 검사를 한다.8MiB capture비용을64KiB라고 표현하면 안 된다. synthetic child stdout64KiB초과 및stderr64KiB초과/descendant lingering timeout 회귀가 후속이다.
4. **원 pin race**: initial verify_package의 각original file pin은 함수 반환 시 사라지고(:186–226), selector는 이후directory/main/probe pin을 새로 연다(:90–94). child후 각identity 확인(:125–129)과 전체rehash는replacement/mutation을 거절하지만 최초 검증과 pin 사이·process executable pathname lookup은원자적이 아니다. pin을검증이전부터유지하고 retainedcap을동일content검증에 사용하는 것은 강화가능하나 동일사용자 공격에대한atomicexec proof가 아니다. metadata pins 역시 verify중에만 유지된다.
5. **role/target**: package.validate 및 tool==ffmpeg, hostcompatible target, exactffmpeg Executable/ffprobe Auxiliary 검사를 한다(:65–78,141–148). Windows MSVC/GNU 둘 다 x86_64호환 후보이며실제Windows 실행검증은 별도이다. package validation의 Executable1개정책이 main identity를 제한한다. wrong tool/role/target/추가foreign 실제거절 회귀를 추가할 수 있다.
6. **cleanup/proof 범위**: run_metadata의 소유 프로세스 취소·타임아웃·drain을 재사용한다. 기존 합성선택회귀는argv[-version], 두success버전 및probe mismatch/nonzero/mutation/foreign/precancel의prioractive 보존을 확인하는 설계다. guardian native descendants의전체종료를이선택unit만으로증명하지 않는다. version stdout은선택package bytes가해당문자열을출력했다는증거이며공급서명/source/license/codec/최소OS 신뢰증거가아니다.

## 인수 조건과 handoff

읽기 감사 및 파일라인 근거 기록 완료. 위1/2를 root에 즉시 메시지로 전달했다. 재현 실행/제품 수정은 source freeze 해제 후 root 조정 대상이다. 공급 후보는 docs/validation/ffmpeg-supply-validation.md의 trust/license/3OS gate를 유지한다.
