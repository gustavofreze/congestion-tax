* [Congestion tax calculating](#congestion-tax-calculating)

## Congestion tax calculating

#### Calculates the tax a vehicle owes, per day and in total, for its passages through the tolling stations of a city.

The passages are grouped by local day, and each day follows the tax period of the city in force on it. On a charged day,
the first charged passage opens a single charge window that covers every charged passage up to 60 minutes after it,
inclusive, and charges the highest fee among them. A free passage never opens a window. Each day is capped at the daily
maximum of its period, and exempt vehicles, weekends, public holidays, the days before them, and July owe nothing in
Gothenburg.

**POST** `{{congestion-tax-dns}}/congestion-taxes`

### Headers

| Header         |  Type  | Description               | Constraints               | Required |
|:---------------|:------:|:--------------------------|:--------------------------|:--------:|
| `Content-Type` | String | The request content type. | Must be application/json. |   Yes    |

### Request

**Body parameters**:

| Parameter  |  Type  | Description                                                                                    | Constraints                                                                                | Required |
|:-----------|:------:|:-----------------------------------------------------------------------------------------------|:-------------------------------------------------------------------------------------------|:--------:|
| `city`     | String | City whose tax rules apply, matched without regard to case or surrounding spaces.              | Must not be blank. E.g., `gothenburg`.                                                     |   Yes    |
| `vehicle`  | String | Type of the vehicle that made the passages.                                                    | Must be one of `BUS`, `CAR`, `DIPLOMAT`, `EMERGENCY`, `FOREIGN`, `MILITARY`, `MOTORCYCLE`. |   Yes    |
| `passages` | Array  | Local dates and times of the city at which the vehicle passed a tolling station, in any order. | Each must be a local date and time in the `YYYY-MM-DD HH:MM:SS` format, with no offset.    |   Yes    |

```json
{
    "city": "gothenburg",
    "vehicle": "CAR",
    "passages": [
        "2013-02-07 06:23:27",
        "2013-02-07 15:27:00"
    ]
}
```

### Response

- `200 OK`

  **Description**: Indicates that the tax was calculated for every day of the passages.

  **Content-Type**: application/json

  **Body**:
  ```json
  {
      "days": [
          {
              "date": "2013-02-07",
              "amount": 21
          }
      ],
      "amount": 21,
      "currency": "SEK"
  }
  ```

- `404 Not Found`

  **Description**: Indicates that the city has no tax rules.

  **Content-Type**: application/json

  **Body**:
  ```json
  {
      "code": "TAX_RULES_NOT_FOUND",
      "message": "No tax rules found for the city."
  }
  ```

- `405 Method Not Allowed`

  **Description**: Indicates that the path exists but does not accept the request method.

  **Content-Type**: application/json

  **Body**:
  ```json
  {
      "code": "METHOD_NOT_ALLOWED",
      "message": "Method Not Allowed."
  }
  ```

- `422 Unprocessable Entity`

  **Description**: Indicates that one or more of the provided values are invalid.

  **Content-Type**: application/json

  **Body**:
  ```json
  {
      "code": "INVALID_REQUEST",
      "message": "Failed to deserialize the JSON body into the target type: missing field `city` at line 1 column 31"
  }
  ```

  or when the city is blank:

  ```json
  {
      "code": "CITY_CANNOT_BE_EMPTY",
      "message": "A city cannot be empty."
  }
  ```

  or when the vehicle is not a supported type:

  ```json
  {
      "code": "UNSUPPORTED_VEHICLE_TYPE",
      "message": "Vehicle type <TRUCK> is not supported."
  }
  ```

  or when a passage is not a local date and time:

  ```json
  {
      "code": "PASSAGE_FORMAT_NOT_VALID",
      "message": "Passage <2013-02-30 07:00:00> is not a local date and time in the YYYY-MM-DD HH:MM:SS format."
  }
  ```

  or when no tax period of the city covers the day of a passage:

  ```json
  {
      "code": "PASSAGE_OUTSIDE_TAX_PERIODS",
      "message": "No tax period of the city covers the passages of <2014-02-04>."
  }
  ```

- `503 Service Unavailable`

  **Description**: Indicates that the service is not ready to handle requests.

  **Content-Type**: application/json

  **Body**:
  ```json
  {
      "code": "SERVICE_UNAVAILABLE",
      "message": "The service is not ready."
  }
  ```
