# S00: private bus 실제 접근성 트리·reader 검증

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11, S00
- 담당/수정 범위: diagnostic_cache; 신규 scripts/reader_bus_probe.py·전용 docs/validation/reader-validation.md/evidence와이계약, 신규 .local-tools/reader 도구
- 선행 작업: 최신제품IcedIME실증/eguiAT-SPI증거, rootprivatebus·project-onlyreader설치승인

## 사용자 결과

실제제품Iced와egui의AT-SPI tree/role/name/state/focus를비교하고가능하면음성없는actualreader가label/focus navigation을처리하는지확인한다.

## 계약과 범위

ownedXvfb/privateD-Bus/privateAT-SPI/tempXDG/GSettings memory만사용한다. 사용자bus/desktop/audio설정·시스템설치금지. 기존probe는read-only재사용, 제품/Cargo/vendor는root소유. 공식Ubuntu deb를 .local-tools/reader에만추출하고hash/license/cost기록. tree공개와actualreader는구별한다.

## 인수 조건

exactbinaryhash별tree/한국어label/role/state/focus증거, no-audio설정·reader로그/탐색이가능하면actual동작증거, 소유process회수·상한·미검증OS를기록한다. Icedtree없음은실제bus조건에서확인하고후보stack의gate결론을전달한다.

## 검증과 handoff

설치된tool조사: at-spi2-core/GI/dbus있음,Orca/pyatspi/speechbackend없음. project-only package준비와boundedtree비교진행.

## 완료 결과

공식 Ubuntu 12개 deb를 프로젝트 전용으로 추출해 실제 Orca 무음 AT-SPI 이벤트 처리 비교를 수행했다. 제품 Iced tree/reader gate FAIL, egui spike의 label·text·focus·Tab 이벤트 처리 확인. 전체 음성/reader 사용성은 미검증이다. 원 collector 지표의 범위 오류와 Iced reader 종료 deadline 실패를 숨기지 않고 review.json으로 구분했다. 전용 검증 문서: docs/validation/reader-validation.md. 제품/Cargo 변경 없음.
