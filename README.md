# общее описание
опция 1 - YpBankCsv,  YpBankText, YpBankBin
Проект состоит из двух модулей - cli ( общая бизнес логика ) и lib ( утилитные и дата юниты )

# cli
Представляет из себя возможность вызвать сравнение двух данных заданных форматов или переовд одного формата в другой
Чтобы получить информацию о работе требуется ввесь ./path/to/bin --help

# lib
Содержит основные структуры и логику парсинга


# примеры команд ( все тестовые файлы содержатся в директории test-data)
./target/debug/cli compare-command --first-from file:records_example.csv --first-format yp-bank-csv --second-from file:records_example.csv --second-format yp-bank-csv
./target/debug/cli compare-command --first-from file:records_example.bin --first-format yp-bank-bin --second-from file:records_example.bin --second-format yp-bank-bin
./target/debug/cli compare-command --first-from file:records_example.txt --first-format yp-bank-text --second-from file:records_example.txt --second-format yp-bank-text



./target/debug/cli read-parse-write-command --from file:records_example.csv --from-format yp-bank-csv --to console --to-format yp-bank-text
./target/debug/cli read-parse-write-command --from file:records_example.csv --from-format yp-bank-csv --to console --to-format yp-bank-bin
./target/debug/cli read-parse-write-command --from file:records_example.csv --from-format yp-bank-csv --to console --to-format yp-bank-csv


# PS 
Я прекрасно осведомлен о некоторых архитектурных проблемах данного решения. Есть множество неоптимальных вызовов и структур. Из-за нехватки времени пришлось пожертвовать качеством. В будущих проектах я исправлю